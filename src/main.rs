use std::sync::Arc;

use arrow::util::pretty;
use datafusion::error::Result;
use datafusion::execution::context::ExecutionContext;
use log::*;
use rustyline::*;
use rustyline::error::ReadlineError;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    let mut ctx = ExecutionContext::new();

    let config = Config::builder()
        .history_ignore_space(true)
        .max_history_size(1024)
        .auto_add_history(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .output_stream(OutputStreamType::Stdout)
        .build();
    let mut rl = Editor::<()>::with_config(config);
    let history = ".dlsh_history";

    if rl.load_history(history).is_err() {
        warn!("No history file present {}", history);
    }

    loop {
        let readline = rl.readline("dlsh> ");
        match readline {
            Ok(line) => {
                if line.starts_with("load") {
                    let parts: Vec<&str> = line.split(' ').collect();
                    if parts.len() != 4 {
                        println!("The `load` command requires three parameters, e.g. `load path as table_name`");
                        break;
                    }

                    let (path, name) = (parts[1], parts[3]);
                    match deltalake::open_table(path).await {
                        Ok(table) => {
                            match ctx.register_table(name, Arc::new(table)) {
                                Ok(_) => {},
                                Err(e) => {
                                    println!("Fail: {:?}", e);
                                },
                            }
                        },
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                }
                else {
                    match ctx.sql(&line) {
                        Ok(query) => {
                            match query.collect().await {
                                Ok(res) => {
                                    pretty::print_batches(&res);
                                },
                                Err(e) => {
                                    println!("Failed to execute the query: {:?}", e);
                                },
                            }
                        },
                        Err(e) => {
                            println!("Failed to process query: {:?}", e);
                        }
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                break;
            },
            Err(ReadlineError::Eof) => {
                break;
            },
            Err(e) => {
                error!("Failed on something: {}", e);
                break;
            },
        }
    }
    rl.save_history(history).expect("Failed to save history");
    Ok(())
}
