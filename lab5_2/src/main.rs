use std::{
    io::{self, BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::{mpsc::channel, Arc},
    thread,
};

enum TerminalEvent {
    UserInput(String),
    UserTermination, // Ctrl-a
    CommandOutput(String),
    CommandEnded,
}

enum TerminalMode {
    Normal,
    Command,
}

fn main() -> io::Result<()> {
    let stdin = Arc::new(io::stdin());
    let mut stdout = io::stdout();

    let (evt_snd, evt_rcv) = channel::<TerminalEvent>();

    let input_sender = evt_snd.clone();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        match stdin.read_line(&mut buffer) {
            Ok(0) => {
                input_sender.send(TerminalEvent::UserTermination);
                break;
            }
            Ok(n) => {
                let term = buffer.chars().find(|c| *c == '\u{1}').is_some();
                input_sender.send(TerminalEvent::UserInput(buffer));
                if term {
                    input_sender.send(TerminalEvent::UserTermination);
                }
            }
            Err(e) => {
                println!("ERR {}", e);
                break;
            }
        }
    });

    let mut mode = TerminalMode::Normal;
    let mut child: Option<Child> = None;

    loop {
        match mode {
            TerminalMode::Command => match evt_rcv.recv() {
                Ok(TerminalEvent::UserTermination) => {
                    if let Some(ref mut child) = child {
                        child.kill().unwrap();
                    }
                }

                Ok(TerminalEvent::UserInput(s)) => {
                    if let Some(ref mut child) = child {
                        if let Some(ref mut stdin) = child.stdin {
                            stdin.write_all(s.as_bytes()).unwrap();
                        }
                    }
                }

                Ok(TerminalEvent::CommandOutput(s)) => {
                    print!("{}", s);
                    stdout.flush();
                }
                Ok(TerminalEvent::CommandEnded) => {
                    mode = TerminalMode::Normal;
                }
                _ => {}
            },

            TerminalMode::Normal => {
                print!("> ");
                stdout.flush();

                match evt_rcv.recv() {
                    Ok(TerminalEvent::UserInput(s)) => {
                        let args: Vec<&str> = s.split_whitespace().collect();
                        child = Some(
                            match Command::new(args[0])
                                .args(&args[1..])
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped())
                                .spawn()
                            {
                                Ok(child) => child,
                                Err(e) => {
                                    println!("ERR {}", e);
                                    continue;
                                }
                            },
                        );
                        mode = TerminalMode::Command;

                        let mut child_output = child.as_mut().unwrap().stdout.take().unwrap();
                        let child_sender = evt_snd.clone();
                        thread::spawn(move || {
                            let mut reader = BufReader::new(child_output);
                            loop {
                                let mut line = String::new();

                                match reader.read_line(&mut line) {
                                    Ok(0) => {
                                        child_sender.send(TerminalEvent::CommandEnded);
                                        break;
                                    }
                                    Ok(n) => {
                                        child_sender.send(TerminalEvent::CommandOutput(line));
                                    }
                                    Err(e) => {
                                        println!("ERR {}", e);
                                        break;
                                    }
                                }
                            }
                        });
                    }

                    Ok(TerminalEvent::UserTermination) => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
