//! Processo e sistema: endurecimento, instância única, pastas, tema, memória, registro.

use super::util::{from_wide, last_error, wide};
use super::{KnownFolder, SystemTheme};
use std::path::PathBuf;
use windows_sys::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, HANDLE, HWND};
use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::System::LibraryLoader::{
    LOAD_LIBRARY_SEARCH_SYSTEM32, SetDefaultDllDirectories,
};
use windows_sys::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX};
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ, RRF_RT_REG_DWORD, RRF_RT_REG_SZ, RegCloseKey,
    RegDeleteValueW, RegGetValueW, RegOpenKeyExW, RegSetValueExW,
};
use windows_sys::Win32::System::RemoteDesktop::{
    NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification,
};
use windows_sys::Win32::System::Threading::{
    CreateMutexW, GR_GDIOBJECTS, GR_USEROBJECTS, GetCurrentProcess, GetGuiResources,
};
use windows_sys::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
use windows_sys::Win32::UI::Shell::{
    FOLDERID_LocalAppData, FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, SHGetKnownFolderPath,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, PostMessageW, RegisterWindowMessageW, SPI_GETHIGHCONTRAST, SystemParametersInfoW,
};

/// Restringe a busca de DLLs carregadas dinamicamente à pasta System32 (mitiga sequestro de DLL
/// quando o executável roda de uma pasta com DLLs maliciosas ao lado). Chamar no início do `main`.
pub fn harden_process() -> bool {
    // SAFETY: sem pré-condições.
    unsafe { SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32) != 0 }
}

/// Mutex de instância única (fechado no `Drop`).
#[derive(Debug)]
pub struct InstanceGuard(HANDLE);

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        // SAFETY: handle criado por `CreateMutexW`.
        unsafe { CloseHandle(self.0) };
    }
}

/// Tenta adquirir a instância única. `None` se outra instância já está rodando nesta sessão.
pub fn single_instance(id: &str) -> Option<InstanceGuard> {
    // Prefixo Local\: escopo da sessão do usuário (troca rápida de usuário permite uma instância
    // por usuário).
    let name = wide(&format!("Local\\{id}.Instance"));
    // SAFETY: atributos nulos; nome terminado em zero.
    let h = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
    let err = last_error();
    if h.is_null() {
        // Sem mutex não há como garantir instância única; o jogo segue (melhor que não abrir).
        return Some(InstanceGuard(std::ptr::null_mut()));
    }
    if err == ERROR_ALREADY_EXISTS {
        // SAFETY: handle válido recém-obtido.
        unsafe { CloseHandle(h) };
        return None;
    }
    Some(InstanceGuard(h))
}

/// Pede à instância existente que se mostre.
pub fn notify_existing_instance(class_prefix: &str) {
    let class = wide(&format!("{class_prefix}.Strip"));
    // SAFETY: string terminada em zero.
    let hwnd = unsafe { FindWindowW(class.as_ptr(), std::ptr::null()) };
    if hwnd.is_null() {
        return;
    }
    let msg_name = wide(&super::window::show_instance_message_name(class_prefix));
    // SAFETY: string terminada em zero.
    let msg = unsafe { RegisterWindowMessageW(msg_name.as_ptr()) };
    if msg != 0 {
        // SAFETY: mensagem registrada, sem ponteiros nos parâmetros.
        unsafe { PostMessageW(hwnd, msg, 0, 0) };
    }
}

/// Caminho de uma pasta conhecida (não usa variáveis de ambiente, que podem estar ausentes).
pub fn known_folder(which: KnownFolder) -> Option<PathBuf> {
    let id = match which {
        KnownFolder::RoamingAppData => &FOLDERID_RoamingAppData,
        KnownFolder::LocalAppData => &FOLDERID_LocalAppData,
    };
    let mut ptr: *mut u16 = std::ptr::null_mut();
    // SAFETY: GUID estático; `ptr` recebe memória alocada pelo shell, liberada abaixo.
    let hr =
        unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT as u32, std::ptr::null_mut(), &mut ptr) };
    if hr < 0 || ptr.is_null() {
        if !ptr.is_null() {
            // SAFETY: memória do shell.
            unsafe { CoTaskMemFree(ptr.cast()) };
        }
        return None;
    }
    let mut len = 0usize;
    // SAFETY: string terminada em zero devolvida pelo shell; a leitura para no terminador.
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }
    // SAFETY: `len` elementos válidos a partir de `ptr`.
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    // Mantém o caminho em UTF-16 (OsString) sem conversão com perda: nomes com acento funcionam.
    let os: std::ffi::OsString = std::os::windows::ffi::OsStringExt::from_wide(slice);
    // SAFETY: memória alocada pelo shell com CoTaskMemAlloc.
    unsafe { CoTaskMemFree(ptr.cast()) };
    Some(PathBuf::from(os))
}

fn reg_dword(subkey: &str, value: &str) -> Option<u32> {
    let k = wide(subkey);
    let v = wide(value);
    let mut data: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;
    // SAFETY: strings terminadas em zero; `data`/`size` são buffers locais coerentes.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            k.as_ptr(),
            v.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&mut data as *mut u32).cast(),
            &mut size,
        )
    };
    (r == 0).then_some(data)
}

/// Tema atual: barra clara ou escura e alto contraste.
pub fn theme() -> SystemTheme {
    let light = reg_dword(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
        "SystemUsesLightTheme",
    )
    .is_some_and(|v| v != 0);
    let mut hc = HIGHCONTRASTW {
        cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32,
        ..Default::default()
    };
    // SAFETY: `hc` é válido e `cbSize` está preenchido.
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            hc.cbSize,
            (&mut hc as *mut HIGHCONTRASTW).cast(),
            0,
        )
    };
    SystemTheme {
        light_taskbar: light,
        high_contrast: ok != 0 && (hc.dwFlags & HCF_HIGHCONTRASTON) != 0,
    }
}

/// Registra a janela para receber bloqueio/desbloqueio de sessão.
pub fn register_session_notifications(hwnd: HWND) {
    // SAFETY: janela desta thread. O registro é removido automaticamente quando a janela é
    // destruída.
    unsafe { WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) };
}

/// Estatísticas de memória e recursos do processo (overlay de depuração e soak test).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MemoryStats {
    /// Private Bytes.
    pub private_bytes: usize,
    /// Working set.
    pub working_set: usize,
    /// Objetos GDI.
    pub gdi_objects: u32,
    /// Objetos USER.
    pub user_objects: u32,
}

/// Lê as estatísticas de memória do processo.
pub fn memory_stats() -> MemoryStats {
    // SAFETY: pseudo-handle do processo atual (não precisa ser fechado).
    let process = unsafe { GetCurrentProcess() };
    let mut pmc = PROCESS_MEMORY_COUNTERS_EX {
        cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
        ..Default::default()
    };
    // SAFETY: a estrutura estendida começa com PROCESS_MEMORY_COUNTERS e `cb` indica o tamanho.
    let ok = unsafe {
        GetProcessMemoryInfo(
            process,
            (&mut pmc as *mut PROCESS_MEMORY_COUNTERS_EX).cast(),
            pmc.cb,
        )
    };
    // SAFETY: pseudo-handle válido.
    let gdi = unsafe { GetGuiResources(process, GR_GDIOBJECTS) };
    // SAFETY: idem.
    let user = unsafe { GetGuiResources(process, GR_USEROBJECTS) };
    MemoryStats {
        private_bytes: if ok != 0 { pmc.PrivateUsage } else { 0 },
        working_set: if ok != 0 { pmc.WorkingSetSize } else { 0 },
        gdi_objects: gdi,
        user_objects: user,
    }
}

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

/// Valor atual da chave "iniciar com o Windows" para `name`.
pub fn run_key_get(name: &str) -> Option<String> {
    let k = wide(RUN_KEY);
    let v = wide(name);
    let mut buf = [0u16; 1024];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: strings terminadas em zero; `buf`/`size` coerentes (tamanho em bytes).
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            k.as_ptr(),
            v.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buf.as_mut_ptr().cast(),
            &mut size,
        )
    };
    (r == 0).then(|| from_wide(&buf))
}

/// Grava (`Some`) ou remove (`None`) o valor "iniciar com o Windows". O caminho é gravado entre
/// aspas. Retorna `true` em caso de sucesso.
pub fn run_key_set(name: &str, exe: Option<&std::path::Path>) -> bool {
    let k = wide(RUN_KEY);
    let mut key: HKEY = std::ptr::null_mut();
    // SAFETY: subchave terminada em zero; `key` recebe o handle, fechado abaixo.
    let r = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, k.as_ptr(), 0, KEY_SET_VALUE, &mut key) };
    if r != 0 {
        return false;
    }
    let v = wide(name);
    let ok = match exe {
        Some(path) => {
            let data = wide(&format!("\"{}\"", path.display()));
            // SAFETY: `key` aberto com KEY_SET_VALUE; `data` é REG_SZ terminado em zero e o tamanho
            // em bytes inclui o terminador.
            unsafe {
                RegSetValueExW(
                    key,
                    v.as_ptr(),
                    0,
                    REG_SZ,
                    data.as_ptr().cast(),
                    (data.len() * 2) as u32,
                ) == 0
            }
        }
        // SAFETY: `key` aberto com KEY_SET_VALUE.
        None => unsafe { RegDeleteValueW(key, v.as_ptr()) == 0 },
    };
    // SAFETY: handle aberto acima.
    unsafe { RegCloseKey(key) };
    ok
}

/// Verdadeiro se o idioma da interface do Windows é português.
pub fn ui_language_is_portuguese() -> bool {
    // SAFETY: sem pré-condições.
    let langid = unsafe { GetUserDefaultUILanguage() };
    // Idioma primário = 10 bits baixos; LANG_PORTUGUESE = 0x16.
    langid & 0x3ff == 0x16
}
