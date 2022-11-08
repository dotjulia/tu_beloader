use gui::start_gui;
use clap::Parser;
use reqwest::blocking::Client;

use crate::api::Value;

mod api;
mod gui;

#[macro_use]
extern crate serde_derive;

#[derive(Parser)]
#[command(version, about = "A quick hack to get direct links to lectures more easily. Specify no argument to open gui or specify either search or id.")]
struct Cli {
    /// Search for series
    #[arg(short, long, value_name = "search")]
    search: Option<String>,
    /// Get episodes for series with id
    #[arg(short, long, value_name = "id")]
    id: Option<String>,
    /// Login using username and password
    #[arg(short, long, value_name = "username")]
    username: Option<String>,
    /// Login using username and password
    #[arg(short, long, value_name = "password")]
    password: Option<String>,
    /// OTP token if enabled
    #[arg(short, long, value_name = "otp_token")]
    otp_token: Option<String>,
}

fn are_args_provided(cli: &Cli) -> bool {
    cli.search.is_some() || cli.id.is_some() || cli.username.is_some() || cli.password.is_some()
}

fn cli_mode(client: Client, cli: &Cli) {
    if cli.username.is_some() {
        if cli.password.is_none() {
            println!("Please provide password");
            return
        }
        let otp_token = match cli.otp_token {
            Some(ref otp_token) => otp_token,
            None => "",
        };
    match api::login(&client, &cli.username.as_ref().unwrap(), &cli.password.as_ref().unwrap(), otp_token) {
            Ok(_) => (),
            Err(e) => {
                println!("Error: {}", e);
                return
            }
        }
    }
    if cli.search.is_some() {
        match api::search_series(&client, &cli.search.as_ref().unwrap()) {
            Ok(res) => {
                for series in res.catalogs {
                    println!("{}: {} - {}",
                        series.body.identifier[0].value,
                        series.body.title[0].value,
                        series.body.creator.unwrap_or(vec![Value { value: "null".to_owned() }])[0].value,
                    );
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                return
            }
        }
    } else if cli.id.is_some() {
        match api::get_series(&client, &cli.id.as_ref().unwrap()) {
            Ok(res) => {
                for episode in res.search_results.results {
                    println!("{}: {} - {}",
                        episode.title,
                        episode.created,
                        episode.creator.unwrap_or("null".to_owned()),
                    );
                    for media in episode.mediapackage.media.track {
                        if media.video.is_some() {
                            println!("\t{}: {}",
                                media.video.unwrap().resolution,
                                media.url,
                            );
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                return
            }
        }
    }
}

fn main() {
    let client = reqwest::blocking::Client::builder().cookie_store(true).build().unwrap();
    let cli = Cli::parse();
    if are_args_provided(&cli) {
        cli_mode(client, &cli)
    } else {
        start_gui(client)
    }
}
