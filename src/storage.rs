extern crate task_scheduler;

use super::parse_time;
use chrono::Utc;
use serenity::client::Client;
use serenity::prelude::Context;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Error, Read, Write};
use std::thread;
use std::time::Duration;
use task_scheduler::Scheduler;

pub fn save_reminder(
    timestamp: i64,
    time_to_wait: i32,
    user_id: u64,
    remind_msg: String,
) -> Result<(), Error> {
    let save_entry = format!(
        "{} {} {} {}",
        timestamp.to_string(),
        time_to_wait.to_string(),
        user_id.to_string(),
        remind_msg
    );

    let save_entry = save_entry.replace("\n", "/n");
    let save_entry = format!("{}\n", save_entry);

    println!("* Save entry --> {}", save_entry);

    let mut file = OpenOptions::new()
        .append(true)
        .open("data.txt")
        .expect("cannot open file");

    file.write_all(save_entry.as_bytes())
        .expect("Storage write failed.");

    Ok(())
}

pub fn load_reminders(client: Client) -> Result<(), Error> {
    println!("* Try load reminders list.");
    use chrono::prelude::*;
    let path = "data.txt";
    use std::sync::{Arc, Mutex};

    let http = Arc::new(Mutex::new(&client.cache_and_http.http));
    let client = Arc::new(Mutex::new(&client));

    if (fs::metadata(path).is_ok()) {
        let mut file = File::open(path).expect("File open failed");
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let scheduler = Scheduler::new();

        let split_args = contents.split("\n").map(|x| x.to_string());

        File::create(path).expect("Storage create failed.");

        for rem in split_args {
            let cloned_client = Arc::clone(&client);
            let cloned_http = Arc::clone(&http);

            if (rem.len() > 8) {
                // println!("Loaded reminder {}", &rem.as_str());
                let mut splitter = rem.splitn(4, " ").map(|x| x.to_string());

                let timestamp = splitter.next().unwrap().parse::<i64>().unwrap();
                let time_to_wait_in_seconds =
                    splitter.next().unwrap().parse::<i32>().unwrap() as i64;
                let user_id = splitter.next().unwrap().parse::<u64>().unwrap();
                let remind_msg = splitter.next().unwrap();

                // From https://stackoverflow.com/a/50072164/13169611
                let naive = NaiveDateTime::from_timestamp(timestamp, 0);
                let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

                let time_since_message = Utc::now().signed_duration_since(datetime).num_seconds();

                // println!("Maybe remind user {} about {}", user_id, remind_msg);

                if (time_since_message < time_to_wait_in_seconds) {
                    let final_time_wait = (time_to_wait_in_seconds - time_since_message) as u64;
                    if (final_time_wait > 0) {
                        save_reminder(
                            timestamp,
                            time_to_wait_in_seconds as i32,
                            user_id,
                            remind_msg.to_string(),
                        );
                        // thread::spawn(move || {
                        scheduler.after_duration(Duration::from_secs(final_time_wait), move || {
                            println!("Remind user {} about {}", user_id, remind_msg);

                            use super::events::HandlerEmpty;
                            use serenity::http::Http;
                            let mut file = File::open(".token").expect("Error opening token file");
                            let mut token = String::new();
                            file.read_to_string(&mut token)
                                .expect("Token could not be read");
                            use parking_lot::RawRwLock;
                            use parking_lot::RwLock;
                            use serenity::cache::Cache;
                            use serenity::cache::CacheRwLock;

                            let rwlock = RwLock::new(Cache::new());
                            let cache_lock = CacheRwLock::from(Arc::new(rwlock));

                            // let mut client =
                            //     Client::new(&token, HandlerEmpty).expect("Error creating client");

                            // client.start().expect("Could not start client.");

                            // let new_http = Http::new_with_token(token.as_str());
                            // let new_http = cloned_client.lock().unwrap().cache_and_http.http; <<- This
                            // let new_http = http.lock().unwrap();
                            // new_http
                            //     .get_upcoming_maintenances()
                            //     .expect("Failed to fetch upcoming maintenance");

                            // let unlocked_ctx = cloned_ctx.lock().unwrap();
                            // let newer_http = Arc::new(Mutex::new(&new_http));
                            // let dm_reminder = new_http
                            //     .get_user(user_id)
                            //     .expect("Failed to retrieve user from id")
                            //     // .direct_message((&cache_lock, &new_http), move |m| {
                            //     .direct_message(new_http, move |m| m.content(remind_msg));
                        });
                        // thread::sleep(std::time::Duration::new(final_time_wait, 0));
                    }
                }
            }
        }
    } else {
        File::create(path).expect("Storage create failed.");
    }

    Ok(())
}
