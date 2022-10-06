use crate::command::{Command, CommandList, Conjunction};

#[derive(Default)]
pub struct Executor {
    pub history: Vec<Command>,
    pub last_status: i32,
}

impl Executor {
    pub fn new() -> Self {
        Executor { history: Vec::new(), last_status: 0 }
    }

    pub fn execute(&mut self, commands: CommandList) -> i32 {
        for command in &commands.0 {
            match command.conj {
                None                       => self.last_status = command.execute(),
                Some(Conjunction::SemiCol) => self.last_status = command.execute(),
                Some(Conjunction::And)     => {
                    if self.last_status == 0 {
                        self.last_status = command.execute();
                    }
                    else {
                        break;
                    }
                },
                Some(Conjunction::Or)      => {
                    //println!("{:?}", command);
                    if self.last_status == 0 {
                        break;
                    }
                    else {
                        self.last_status = command.execute();
                    }
                }
                Some(_)                    => self.last_status = command.execute(),
            }
        }
        self.last_status
    }
}
