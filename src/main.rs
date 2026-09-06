use std::process;

use alc::{
    MeetingRequest, Request, daily_reflection, meeting, parse_request_with_today, theme::Theme,
};
use chrono::{Local, Utc};
use chrono_tz::America::New_York;

fn main() {
    let args = std::env::args_os().collect::<Vec<_>>();

    if args.len() == 1 {
        print_help();
        return;
    }

    let request = parse_request_with_today(args, || Local::now().date_naive())
        .unwrap_or_else(|error| error.exit());
    match request {
        Request::DailyReflection(date) => print_reflection(date),
        Request::Meeting(request) => print_meetings(request),
    }
}

fn print_help() {
    let mut command = alc::command();
    if let Err(error) = command.print_help() {
        eprintln!(
            "{}",
            Theme::active().error_line("✗", &format!("could not print help: {error}"))
        );
        process::exit(1);
    }
}

fn print_reflection(date: daily_reflection::ReflectionDate) {
    let theme = Theme::active();
    eprintln!(
        "{} {}",
        theme.info("●"),
        theme.muted(&format!("Fetching Daily Reflection for {date}..."))
    );
    match daily_reflection::fetch(date) {
        Ok(reflection) => print!("{}", daily_reflection::render(&reflection, theme)),
        Err(error) => {
            eprintln!("{}", theme.error_line("✗", &error.to_string()));
            process::exit(1);
        }
    }
}

fn print_meetings(request: MeetingRequest) {
    let theme = Theme::active();
    eprintln!(
        "{} {}",
        theme.info("●"),
        theme.muted("Loading New York Inter-Group meetings...")
    );
    let at = Utc::now().with_timezone(&New_York);
    let today = at.date_naive();
    let page_url = std::env::var("ALC_NYIG_URL").ok();
    let result = page_url.as_deref().map_or_else(
        || meeting::fetch_cached(today),
        |page_url| meeting::fetch_cached_page(page_url, today),
    );
    let meetings = match result {
        Ok(meetings) => meetings,
        Err(error) => {
            eprintln!("{}", theme.error_line("✗", &error.to_string()));
            process::exit(1);
        }
    };

    match request {
        MeetingRequest::Now(options) => {
            let matches = meeting::now(&meetings, at, &options);
            meeting::display_now(&matches, options.origin(), options.pager_enabled(), theme)
        }
        MeetingRequest::Find(filters) => {
            let matches = meeting::find(&meetings, &filters);
            meeting::display_find(
                &matches,
                filters.origin(),
                &at,
                filters.pager_enabled(),
                theme,
            )
        }
        MeetingRequest::Schedule(options) => {
            let matches = meeting::schedule(&meetings, &at, &options);
            meeting::display_find(
                &matches,
                options.origin(),
                &at,
                options.pager_enabled(),
                theme,
            )
        }
    }
    .unwrap_or_else(|error| {
        eprintln!(
            "{}",
            theme.error_line("✗", &format!("could not display meetings: {error}"))
        );
        process::exit(1);
    });
}
