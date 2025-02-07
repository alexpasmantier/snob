#[allow(unused_imports)]
use log::debug;
use std::io::{stdin, BufRead};

pub fn read_from_stdin() -> Vec<String> {
    let mut stdin = stdin().lock();
    let mut buffer = String::new();

    let mut lines = Vec::new();

    loop {
        match stdin.read_line(&mut buffer) {
            Ok(c) if c > 0 => {
                if !buffer.trim().is_empty() {
                    lines.push(buffer.trim().to_string());
                }
                buffer.clear();
            }
            _ => break,
        }
    }

    lines
}

/// Heuristic to determine if stdin is readable.
/// this is useful for the case where users pipe file names into snob directly
pub fn is_readable_stdin() -> bool {
    use std::io::IsTerminal;

    #[cfg(unix)]
    fn imp() -> bool {
        use std::{
            fs::File,
            os::{fd::AsFd, unix::fs::FileTypeExt},
        };

        let stdin = std::io::stdin();
        let Ok(fd) = stdin.as_fd().try_clone_to_owned() else {
            return false;
        };
        let file = File::from(fd);
        let Ok(md) = file.metadata() else {
            return false;
        };
        let ft = md.file_type();
        let is_file = ft.is_file();
        let is_fifo = ft.is_fifo();
        let is_socket = ft.is_socket();
        is_file || is_fifo || is_socket
    }

    #[cfg(windows)]
    fn imp() -> bool {
        let stdin = winapi_util::HandleRef::stdin();
        let typ = match winapi_util::file::typ(stdin) {
            Ok(typ) => typ,
            Err(err) => {
                debug!(
                    "for heuristic stdin detection on Windows, \
                     could not get file type of stdin \
                     (thus assuming stdin is not readable): {err}",
                );
                return false;
            }
        };
        let is_disk = typ.is_disk();
        let is_pipe = typ.is_pipe();
        let is_readable = is_disk || is_pipe;
        debug!(
            "for heuristic stdin detection on Windows, \
             found that is_disk={is_disk} and is_pipe={is_pipe}, \
             and thus concluded that is_stdin_readable={is_readable}",
        );
        is_readable
    }

    #[cfg(not(any(unix, windows)))]
    fn imp() -> bool {
        debug!("on non-{{Unix,Windows}}, assuming stdin is not readable");
        false
    }

    !std::io::stdin().is_terminal() && imp()
}
