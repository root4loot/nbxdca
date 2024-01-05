use clap::{arg, command, value_parser, Command};
use nbxdca::{read_file_contents, Account, Order};
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let mut cfg_fp = "./config.toml".to_string();

    let matches = command!() // requires `cargo` feature
        .arg(
            arg!(-c --config <FILE> "Path to config.toml (Default ./config.toml)")
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .propagate_version(true)
        .subcommand_required(false)
        .arg_required_else_help(true)
        .author("@danielantonsen")
        .about("nbx.com cli interface")
        .help_template(
            //https://docs.rs/clap/2.32.0/clap/struct.App.html#method.template
            "{about} | {author} \n\nUsage: {usage}\n\n{all-args}",
        )
        .arg_required_else_help(true)
        .version("1.0")
        .subcommand(
            Command::new("account")
                .about("Account command")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("balance")
                        .about("get account balance")
                        .arg(arg!([ASSET]).required(true).help("Asset ID")),
                ),
        )
        .subcommand(
            Command::new("order")
                .about("Order command")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("new")
                        .about("place new order")
                        .arg_required_else_help(true)
                        .arg(arg!([SIDE]).required(true).help("Order side (BUY or SELL)"))
                        .arg(
                            arg!([TICKER])
                                .required(true)
                                .help("Ticker symbol (BTC, ETH, etc)"),
                        )
                        .arg(
                            arg!([AMOUNT])
                                .required(true)
                                .help("Amount in fiat currency for BUY orders or quantity in crypto for SELL orders")
                                .value_parser(value_parser!(f64)), // Parse as floating-point number
                        ),
                )
                .subcommand(Command::new("details").about("print details for last order")),
        )
        .get_matches();

    // Default path "./config.toml" or custom path from --config argument
    let cfg_fp = matches
        .get_one::<PathBuf>("config")
        .map_or_else(|| "./config.toml".to_string(), |p| p.display().to_string());

    // Check if the config file exists
    if !PathBuf::from(&cfg_fp).exists() {
        eprintln!("Error: Configuration file '{}' not found.", cfg_fp);
        return Err("Configuration file not found".into());
    }

    let file_contents = read_file_contents(cfg_fp.to_string())?;
    let config: Account = toml::from_str(&file_contents)?;
    let account = Account::new(config);
    let token = account.account_token()?;

    match matches.subcommand() {
        Some(("account", sub_matches)) => match sub_matches.subcommand() {
            Some(("balance", sub_m)) => println!(
                "{:?}",
                account.account_asset_by_id(sub_m.get_one::<String>("ASSET").unwrap(), &token)?
            ),
            _ => {}
        },
        Some(("order", sub_matches)) => match sub_matches.subcommand() {
            Some(("new", sub_m)) => {
                let side = sub_m.get_one::<String>("SIDE").unwrap().to_string();
                let ticker = sub_m.get_one::<String>("TICKER").unwrap().to_string();
                let amount = *sub_m.get_one::<f64>("AMOUNT").unwrap();

                println!(
                    "Create new order: Side: {}, Ticker: {}, Amount: {}",
                    side, ticker, amount
                );
                // Call to Order::create with the new parameters
                match Order::create(&account, &token, &side, &ticker, amount) {
                    Ok(success) => println!("Order created successfully: {}", success),
                    Err(e) => println!("Error creating order: {}", e),
                }
            }
            Some(("details", _sub_m)) => {
                println!("{:?}", Order::details_by_last_id(&account, &token))
            }
            _ => {}
        },
        _ => {}
    };
    Ok(())
}
