use egui::{RichText, Color32, ScrollArea};
use reqwest::blocking::Client;

use crate::api::{self, SeriesRequest};

struct TubeApp {
    client: Client,
    logged: bool,
    username: String,
    password: String,
    login_error: String,
    search_term: String,
    search_results: Option<api::SearchRequest>,
    episodes: Option<SeriesRequest>,
    search_error: String,
    otp_token: String,
}

impl TubeApp {
    fn new(client: Client) -> Self {
        TubeApp {
            client,
            logged: false,
            username: String::new(),
            password: String::new(),
            login_error: String::new(),
            search_term: String::new(),
            search_results: None,
            search_error: String::new(),
            episodes: None,
            otp_token: String::new(),
        }
    }
}

pub fn start_gui(client: Client) {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "TUbe",
        options,
        Box::new(|_cc| Box::new(TubeApp::new(client))),
    )
}

impl eframe::App for TubeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(RichText::new("TUbe Loader").size(30.0));
            if !self.logged {
                ui.label("Login");
                ui.add_space(1.0);
                ui.horizontal(|ui| {
                    ui.label("Username");
                    ui.text_edit_singleline(&mut self.username);
                });
                ui.horizontal(|ui| {
                    ui.label("Password");
                    ui.text_edit_singleline(&mut self.password);
                });
                ui.horizontal(|ui| {
                    ui.label("OTP");
                    ui.text_edit_singleline(&mut self.otp_token);
                });
                if ui.button("Login").clicked() {
                    match api::login(&self.client, &self.username, &self.password, &self.otp_token) {
                        Ok(_) => self.logged = true,
                        Err(e) => {
                            self.login_error = e + "\nSome errors require a restart of the application";
                        },
                    }
                }
                ui.label(RichText::new(&self.login_error).color(Color32::RED));
                ui.add_space(10.0);
            }
            ui.horizontal(|ui| {
                ui.label("Search: ");
                ui.text_edit_singleline(&mut self.search_term);
                if (ui.button("Search").clicked()) && (self.search_term.len() > 0) {
                    match api::search_series(&self.client, &self.search_term) {
                        Ok(s) => {
                            self.search_results = Some(s);
                            self.episodes = None;
                        }
                        Err(e) => self.search_error = e,
                    }
                }
            });
            ui.label(RichText::new(&self.search_error).color(Color32::RED));
            if self.search_results.is_some() {
                ui.label("Search results:");
                ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("search_results").max_col_width(500.0).striped(true).show(ui, |ui| {
                        for series in &self.search_results.as_ref().unwrap().catalogs {
                            let series = &series.body;
                            ui.vertical(|ui| {
                                ui.add(egui::Label::new(&series.title[0].value).wrap(true));
                                if series.creator.is_some() {
                                    ui.label(&series.creator.as_ref().unwrap()[0].value);
                                }
                                ui.label(&series.identifier[0].value);
                                ui.add_space(10.0);
                            });
                            if ui.button("Load Episodes").clicked() {
                                match api::get_series(&self.client, &series.identifier[0].value) {
                                    Ok(s) => {
                                        self.episodes = Some(s);
                                    }
                                    Err(e) => self.search_error = e,
                                }
                            }
                            ui.end_row();
                        }
                    });
                });
                
                if self.episodes.is_some() {
                    self.search_results = None;
                }
            }
            if self.episodes.is_some() {
                ui.label("Episodes:");
                ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("search_results").max_col_width(500.0).striped(true).show(ui, |ui| {
                        for episode in &self.episodes.as_ref().unwrap().search_results.results {
                            ui.vertical(|ui| {
                                ui.add(egui::Label::new(RichText::new(&episode.title).strong()).wrap(true));
                                if episode.creator.is_some() { 
                                    ui.add(egui::Label::new(episode.creator.as_ref().unwrap()).wrap(true));
                                }
                                ui.add(egui::Label::new(&episode.created).wrap(true));
                                ui.add(egui::Label::new(&episode.id).wrap(true));
                            });
                            ui.vertical(|ui| {
                                for media in &episode.mediapackage.media.track {
                                    if media.video.is_some() {
                                        ui.hyperlink_to(&media.video.as_ref().unwrap().resolution, &media.url);
                                    }
                                }
                            });
                            ui.add_space(5.0);
                            ui.end_row();
                        }
                    });
                });
            }
        });
    }
}
