use crate::execute::Execute;
use crate::command::*;
use conch_parser::ast;
use anyhow::Result;

type BoxedExecutable = Box<dyn Execute>;

pub fn generate_command(top_level_command: ast::TopLevelCommand<String>) -> Result<BoxedExecutable> {
    let boxed_command = parse_top_level(&top_level_command)?;
    Ok(boxed_command)
}

fn parse_top_level(top_level_command: &ast::TopLevelCommand<String>) -> Result<BoxedExecutable> {
    parse_command(top_level_command)
}

fn parse_command(command: &ast::Command<ast::AndOrList<ListableCommandAlias>>) -> Result<BoxedExecutable> {
    match command {
        ast::Command::Job(_)       => Err(anyhow!("Implement jobs")),
        ast::Command::List(and_or) => parse_and_or_list(and_or),
    }
}

fn parse_and_or_list(and_or_list: &ast::AndOrList<ListableCommandAlias>) -> Result<BoxedExecutable> {
    let first_unparsed = &and_or_list.first;
    let rest_unparsed = &and_or_list.rest;
    let first = parse_listable(first_unparsed);
    let mut rest: Vec<_> = vec![];
    for and_or_command in rest_unparsed {
        rest.push(parse_and_or_command(and_or_command));
    }
    let boxed_list = Box::new(AndOrCommandList{first, rest});
    Ok(boxed_list)
}

fn parse_listable(listable: &ListableCommandAlias) -> BoxedExecutable {
    match listable {
        ast::ListableCommand::Single(command) => Box::new(build_single_command(command)),
        ast::ListableCommand::Pipe(bang, list) => Box::new(build_pipe_command(*bang, list)),
    }
}

fn parse_and_or_command(and_or_command: &ast::AndOr<ListableCommandAlias>) -> AndOrCommand {
    match and_or_command {
        ast::AndOr::And(command) => build_and_or_command(command, Conjunction::And),
        ast::AndOr::Or(command) => build_and_or_command(command, Conjunction::Or),
    }
}

fn parse_pipeable(pipeable: &PipeableCommandAlias) -> Box<dyn Pipe> {
    match pipeable {
        ast::PipeableCommand::Simple(simple) =>
        {
            let mut args: Vec<String> = vec![];
            let mut redirs: Vec<&ast::Redirect<ast::TopLevelWord<String>>> = vec![];
            let command_words = &simple.redirects_or_cmd_words;
            for word in command_words {
                match word {
                    ast::RedirectOrCmdWord::Redirect(redir) => {
                        redirs.push(redir);
                    },
                    ast::RedirectOrCmdWord::CmdWord(ast::TopLevelWord(complex_word)) => {
                        let string_word = parse_complex_word(complex_word);
                        args.push(string_word);
                    }
                }
            }
            let command: Box<dyn Pipe> = if redirs.len() == 1 {
                    Box::new(build_redirect_command(&args, redirs.pop().unwrap()))
                }
                else {
                    Box::new(build_simple(&args))
                };
            command
        }
        ast::PipeableCommand::Compound(_) => { todo!("Implement compound commands in pipes") },
        ast::PipeableCommand::FunctionDef(_, _) => { todo!("Implement functions in pipes") },
    }

}

fn build_pipe_command(bang: bool, command_list: &Vec<PipeableCommandAlias>) -> PipeCommands {
    let mut commands = vec![];

    for command in command_list {
        commands.push(parse_pipeable(command));
    }
    PipeCommands{ commands, bang }
}

fn build_and_or_command(listable_command: &ListableCommandAlias, conjunction: Conjunction) -> AndOrCommand {
    let command = parse_listable(listable_command);
    AndOrCommand{command, conjunction}
}

fn build_single_command(single: &PipeableCommandAlias) -> SingleCommand {
    let boxed_executable = match single {
        ast::PipeableCommand::Simple(simple) => parse_simple(simple),
        ast::PipeableCommand::Compound(compound) => parse_compound(compound),
        ast::PipeableCommand::FunctionDef(_, _) => { todo!("Implement functions in pipes") },
    };
    SingleCommand { command: boxed_executable }
}

fn parse_simple(simple: &SimpleCommandAlias) -> Box<dyn Execute> {
    let mut args: Vec<String> = vec![];
    let mut redirs: Vec<&ast::Redirect<ast::TopLevelWord<String>>> = vec![];
    let command_words = &simple.redirects_or_cmd_words;
    for word in command_words {
        match word {
            ast::RedirectOrCmdWord::Redirect(redir) => {
                redirs.push(redir);
            },
            ast::RedirectOrCmdWord::CmdWord(ast::TopLevelWord(complex_word)) => {
                let string_word = parse_complex_word(complex_word);
                args.push(string_word);
            }
        }
    }
    let command: BoxedExecutable = if redirs.len() == 1 {
            Box::new(build_redirect_command(&args, redirs.pop().unwrap()))
        }
        else if let Some(builtin) = build_builtin_command(&args) {
            Box::new(builtin)
        }
        else {
            Box::new(build_simple(&args))
        };
    command
}

fn parse_compound(compound: &CompoundCommandAlias) -> Box<dyn Execute> {
    let kind = &compound.kind;
    match kind {
        ast::CompoundCommandKind::Subshell(command_list) => {
            let commands_result: Result<Vec<_>> = command_list.iter().map(|com| parse_top_level(com)).collect();
            let commands = match commands_result {
                Ok(commands) => commands,
                Err(e) => {
                    eprintln!("Error parsing subshell: {e}");
                    return Box::new(BuiltinCommand { args: vec![] });
                },
            };
            Box::new(SubshellCommand { commands })
        }
            _ => todo!("Implement compound commands"),
    }
}

fn build_simple(args: &[String]) -> SimpleCommand {
    let mut command = process::Command::new(args[0].as_str());
    for arg in &args[1..] {
        command.arg(arg);
    }
    SimpleCommand{ command }
}

fn build_redirect_command(args: &[String], redir: &ast::Redirect<ast::TopLevelWord<String>>) -> RedirectCommand {
    let mut command = process::Command::new(args[0].as_str());
    for arg in &args[1..] {
        command.arg(arg.as_str());
    }
    let redirect = convert_redirect(redir);
    RedirectCommand{ command, redirect }
}

fn build_builtin_command(in_args: &[String]) -> Option<BuiltinCommand> {
    let mut args: Vec<String> = vec![];
    if let Some(builtin) = in_args.first() {
        match builtin.as_str() {
            "cd" | "pwd" | "/bin/pwd" | "exec" | "exit" => {
                for arg in in_args {
                    args.push(arg.clone());
                }
                Some(BuiltinCommand{ args })
            },
            _ => None,
        }
    }
    else {
        None
    }
}

fn parse_complex_word(complex_word: &ComplexWordAlias) -> String {
    let string_word: String;
    match complex_word {
        ast::ComplexWord::Concat(_word_list) => {
            eprintln!("Concatenated lists of words aren't supported");
            return String::from("");
        },
        ast::ComplexWord::Single(word) => {
            match word {
                ast::Word::DoubleQuoted(_word_list) => {
                    eprintln!("Double quotes aren't supported");
                    return String::from("");
                },
                ast::Word::SingleQuoted(_word_list) => {
                    eprintln!("Single quotes aren't supported");
                    return String::from("");
                },
                ast::Word::Simple(simple_word) => {
                    match simple_word {
                        ast::SimpleWord::Literal(lit) => { string_word = lit.clone() },
                        ast::SimpleWord::Escaped(esc) => { string_word = esc.clone() },
                        ast::SimpleWord::Subst(param_sub)   => {
                            string_word = convert_subst_to_string(param_sub);
                        }
                        _ => {
                                eprintln!("Unsupported literal");
                                return String::from("");
                            },
                    }
                },
            }
        }
    }
    string_word
}

fn convert_subst_to_string(parameter: &ParameterAlias) -> String {
    let mut string = String::from("");
    match parameter {
        ast::ParameterSubstitution::Command(vec_commands) => {
            for command in vec_commands {
                let executable_result = parse_top_level(command);
                let res_string = match executable_result {
                    Err(e)         => { eprintln!("Error parsing command substitution: {}", e); String::from("") }
                    Ok(mut executable) => match executable.execute_to_string() {
                        Ok(string) => string,
                        Err(e)     => { eprintln!("Error parsing command substitution: {}", e); String::from("") }
                    }
                };
                string.push_str(res_string.trim());
            }
        },
        _ => eprintln!("Unsupported parameter substitution"),
    }
    string
}

fn convert_redirect(redir: &ast::Redirect<ast::TopLevelWord<String>>) -> Redirect {
    match redir {
        ast::Redirect::Read(fd, dest) => {
            let filename = parse_complex_word(dest);
            Redirect::Read(*fd, filename)
        }
        ast::Redirect::Write(fd, dest) => {
            let filename = parse_complex_word(dest);
            Redirect::Write(*fd, filename)
        }
        ast::Redirect::Append(fd, dest) => {
            let filename = parse_complex_word(dest);
            Redirect::Append(*fd, filename)
        }
        ast::Redirect::ReadWrite(fd, dest) => {
            let filename = parse_complex_word(dest);
            Redirect::ReadWrite(*fd, filename)
        }
        ast::Redirect::Clobber(_, _com) => todo!(),
        ast::Redirect::Heredoc(_, _com) => todo!(),
        ast::Redirect::DupRead(_, _com) => todo!(),
        ast::Redirect::DupWrite(_lhs, _rhs) => {
            todo!("Implement FD dup");
        }
    }
}

// ********************************************
// Decompose conch_parser's AST structure into useable type aliases
// ********************************************
type ListableCommandAlias = ast::ListableCommand<PipeableCommandAlias>;

type PipeableCommandAlias =
    ast::PipeableCommand
        <String,
        Box<SimpleCommandAlias>,
        Box<CompoundCommandAlias>,
        std::rc::Rc<CompoundCommandAlias>>;

type SimpleCommandAlias =
    ast::SimpleCommand<String, ast::TopLevelWord<String>, ast::Redirect<ast::TopLevelWord<String>>>;

type CompoundCommandAlias =
    ast::CompoundCommand<ast::CompoundCommandKind<String, ast::TopLevelWord<String>, ast::TopLevelCommand<String>>, ast::Redirect<ast::TopLevelWord<String>>>;

type ComplexWordAlias =
    ast::ComplexWord<ast::Word<String, ast::SimpleWord<String, ast::Parameter<String>, Box<ast::ParameterSubstitution<ast::Parameter<String>, ast::TopLevelWord<String>, ast::TopLevelCommand<String>, ast::Arithmetic<String>>>>>>;

type ParameterAlias =
    ast::ParameterSubstitution<ast::Parameter<String>, ast::TopLevelWord<String>, ast::TopLevelCommand<String>, ast::Arithmetic<String>>;
