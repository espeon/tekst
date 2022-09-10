use std::{
    io::stdout,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use clients::Client;
use crossterm::{
    cursor, execute,
    style::{Color, Print, SetForegroundColor},
    terminal,
};
use structs::Lyrics;

use crate::sources::LyricsSource;

mod clients;
mod parse;
mod sources;
mod state;
mod structs;

fn main() {
    loop {
        let client = clients::spotify::SpotifyClient::init();

        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::Hide
        )
        .unwrap();

        print_info(
            &"Getting metadata from music source.".to_string(),
            &"".to_string(),
        );

        match client.get_metadata() {
            Some(e) => {
                let title = match &e.title {
                    Some(e) => &e,
                    None => "",
                };

                let artist = match (&e.artist, title) {
                    (_, "") => "",
                    (Some(i), _) => &i,
                    (_, _) => "",
                };
                print_info(
                    &"Getting metadata from music source.  ☑".to_string(),
                    &"Getting metadata from lyrics source.".to_string(),
                );
                print_song(title, artist);
                let lyrics = match sources::xmlyr::XmLyrSource::get(e.clone()) {
                    Some(j) => j,
                    None => {
                        print_error(
                            &format!("No lyrics found for track: {} - {}", title, artist),
                            &format!("Checking again in 5 seconds."),
                        );
                        thread::sleep(Duration::from_secs_f32(5.00));
                        continue;
                    }
                };
                print_info(
                    &"Getting metadata from music source.  ☑".to_string(),
                    &"Getting metadata from lyrics source. ☑\n Starting player.".to_string(),
                );
                setup(lyrics, client);
            }
            None => {
                let time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards");
                print_error(
                    &format!("Nothing is currently playing (current time: {:?})", time),
                    &"Checking again in 5 seconds.".to_string(),
                );
                thread::sleep(Duration::from_secs_f32(5.00));
                continue;
            }
        };
    }
}

fn print_error(l1: &String, l2: &String) {
    execute!(
        stdout(),
        cursor::MoveTo(1, 2),
        terminal::Clear(terminal::ClearType::All),
        SetForegroundColor(Color::Red),
        Print("Oh no! Error:"),
        SetForegroundColor(Color::White),
        cursor::MoveTo(1, 4),
        Print(l1),
        cursor::MoveTo(1, 5),
        Print(l2)
    )
    .unwrap();
}

fn print_info(l1: &String, l2: &String) {
    execute!(
        stdout(),
        cursor::MoveTo(1, 2),
        SetForegroundColor(Color::White),
        cursor::MoveTo(1, 4),
        Print(l1),
        cursor::MoveTo(1, 5),
        Print(l2)
    )
    .unwrap();
}

fn print_song(title: &str, artist: &str) {
    let separator = match (title, artist) {
        ("", "") => "",
        (_, _) => "-",
    };

    execute!(
        stdout(),
        cursor::MoveTo(1, 1),
        terminal::Clear(terminal::ClearType::CurrentLine),
        SetForegroundColor(Color::AnsiValue(253)),
        Print(format!("{} {} {}", title, separator, artist)),
    )
    .unwrap();
}

fn setup(lyrics: Lyrics, client: impl Client + Clone) {
    // max time
    let to_elapse = Duration::from_millis(900000);

    // constant delta time
    const DT: u128 = 1 * 1000000; // as nanoseconds (* 1000000)
    let mut accumulator = 00;
    let mut current_time: Instant = Instant::now();
    let mut time = client.get_pos().unwrap().position.unwrap();
    dbg!(&time);

    // debug vars
    let mut update_count = 0;
    let mut frame_count = 0;
    let timer = Instant::now();

    // set default state and terminal look
    let mut state: (Vec<(&String, &Duration)>, usize) = (vec![], 0);
    let mut prev_state: (Vec<(&String, &Duration)>, usize) = (vec![], 0);
    let time_until_update = 5000; // ms
    let mut next_update_time = 0;
    let mut paused: bool;
    let mut to_break = false;
    let mut loading_shown: bool;

    let lines = terminal::size().unwrap_or((132, 20)).1 as usize - 3;

    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide
    )
    .unwrap();

    let title = match &lyrics.metadata.title {
        Some(e) => &e,
        None => "",
    };

    let artist = match (&lyrics.metadata.artist, title) {
        (_, "") => "",
        (Some(i), _) => &i,
        (_, _) => "",
    };

    print_song(title, artist);

    loading_shown = true;

    // loop
    loop {
        let now = Instant::now();
        // get last iteration's loop time
        let mut frame_time = now - current_time;
        // just in case:tm: our logic can't catch up with our frames
        if frame_time.as_secs_f32() > 0.25 {
            frame_time = Duration::from_millis(0250);
        }

        current_time = now;

        // set accumulator
        accumulator += frame_time.as_nanos();

        // if "loading" is shown, we force an update here as
        // otherwise lyrics display after fetching would not
        // be immediate
        if loading_shown == true {
            state = update(&lyrics, time, lines);
            render(&state, &prev_state, lines);
            prev_state = state.clone();
            loading_shown = false;
        }

        // if we can do a tick do it !!!
        // logic loop
        while accumulator >= DT {
            update_count += 1;
            time += Duration::from_nanos(DT.try_into().unwrap());
            if time.as_millis() > next_update_time {
                // get position and metadata for comparing
                // we set position and check metadata against current
                // for song detection
                // TODO: put this in its own separate thread so it doesn't clog up the main thread
                let pos = match client.get_pos(){
                    Some(e) => e,
                    None => break,
                };

                time = pos.position.unwrap_or(time);
                paused = !pos.playing;

                let meta = client.get_metadata().unwrap();

                // simple comparison to check song change and pausing
                if meta.title.clone().unwrap() != lyrics.metadata.title.to_owned().unwrap() {
                    to_break = true;
                }
                // todo: fix this cause it doesn't work now lmao
                if paused && time.as_millis() > next_update_time {
                    time -= Duration::from_millis(next_update_time.try_into().unwrap());
                } else {
                    next_update_time += time_until_update;
                }
            }
            state = update(&lyrics, time, lines);
            accumulator -= DT;
        }

        // if we can exit here, we exit
        // it's better to do it now cause if we were to not
        // it would lead to an extra compare and render later
        if timer.elapsed() > to_elapse || to_break {
            break;
        }

        frame_count += 1;

        if &prev_state != &state {
            render(&state, &prev_state, lines);
            prev_state = state.clone();
        }

        // sleep for .1 seconds (power consumption issue lul)
        thread::sleep(Duration::from_secs_f32(0.1));

        // exit
        if timer.elapsed() > to_elapse || to_break {
            break;
        }
    }

    println!(
        "took {} s (current time: {} | update count: {} | frame count: {} )",
        timer.elapsed().as_secs_f32(),
        time.as_secs_f32(),
        update_count,
        frame_count
    );
}

fn update(ly: &Lyrics, time: Duration, lines: usize) -> (Vec<(&String, &Duration)>, usize) {
    let mut v: Vec<(&String, &Duration)> = vec![];

    let mut i = 0;
    let local_index;

    // get current + 1 spot (first line whose timestamp is greater than current)
    for line in &ly.lines {
        if line.start < time {
            i += 1;
        } else {
            break;
        };
    }
    //dbg!((i, time, &ly.lines[i - 1].line));

    // check if we can display the first line
    // if i is 0, then we don't push (it will error) and set
    // our "current value" we send to the render to the first value
    if i > 1 {
        v.push((&ly.lines[i - 2].line, &ly.lines[i - 2].start));
        local_index = 1;
    } else {
        local_index = 0;
    }

    // probably current active line
    if i > 0 {
        v.push((&ly.lines[i - 1].line, &ly.lines[i - 1].start));
    }
    // next lines
    for j in 0..lines - 2 {
        if i + j < ly.lines.len() {
            v.push((&ly.lines[i + j].line, &ly.lines[i + j].start));
        }
    }

    (v, local_index)
}

fn render(
    lines: &(Vec<(&String, &Duration)>, usize),
    prev_state: &(Vec<(&String, &Duration)>, usize),
    lyrics_height: usize
) {
    // y offset
    let mut i = 3;
    let height = lyrics_height + 3;
    if lines == prev_state {
        return;
    }
    for line in &lines.0 {
        // add current marker to current line
        let tp = match lines.1 == i - 3 {
            true => ("> ".to_string() + line.0),
            _ => line.0.to_string(),
        };
        // set colours
        let color =
            match lines.1 == i - 3 {
                true => Color::AnsiValue(15),
                _ => Color::AnsiValue(
                    (255 - ((std::cmp::max(i, 5) as f32 / std::cmp::max(lines.0.len(), 7) as f32)
                        * 9.0)
                        .log(1.15)
                        .ceil() as usize)
                        .try_into()
                        .unwrap(),
                ),
            };
        // idfk what this does
        execute!(
            stdout(),
            cursor::MoveTo(1, i.try_into().unwrap()),
            terminal::Clear(terminal::ClearType::CurrentLine),
            SetForegroundColor(color),
            Print(tp)
        )
        .unwrap();
        i += 1;
    }
    for _ in i..height {
        execute!(
            stdout(),
            cursor::MoveTo(1, i.try_into().unwrap()),
            terminal::Clear(terminal::ClearType::CurrentLine),
        )
        .unwrap();
        i += 1;
    }
}
