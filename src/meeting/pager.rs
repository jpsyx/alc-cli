use std::io::{self, IsTerminal, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::theme::Theme;

use super::{
    render::{AccessFilter, MeetingList},
    table::truncate_to_width,
};

#[derive(Default)]
pub(super) struct PagerState {
    offset: usize,
    query: String,
    mode: PagerMode,
    access_filter: AccessFilter,
}

#[derive(Default, Eq, PartialEq)]
enum PagerMode {
    #[default]
    Navigate,
    Search,
}

#[derive(Clone, Copy)]
pub(super) enum PagerCommand {
    Down,
    Up,
    HalfDown,
    HalfUp,
    End,
    Start,
    Search,
    Character(char),
    Backspace,
    ClearSearch,
    AcceptSearch,
    All,
    Hybrid,
    InPerson,
    Online,
    Reset,
    Quit,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum PagerOutcome {
    Continue,
    Quit,
}

impl PagerState {
    pub(super) fn apply(
        &mut self,
        command: PagerCommand,
        total_lines: usize,
        viewport_height: usize,
    ) -> PagerOutcome {
        if command_is_quit(command) {
            return PagerOutcome::Quit;
        }

        if self.mode == PagerMode::Search {
            match command {
                PagerCommand::Character(character) => {
                    self.query.push(character);
                    self.offset = 0;
                }
                PagerCommand::Backspace => {
                    if self.query.is_empty() {
                        self.mode = PagerMode::Navigate;
                    } else {
                        self.query.pop();
                    }
                    self.offset = 0;
                }
                PagerCommand::ClearSearch => {
                    self.query.clear();
                    self.offset = 0;
                }
                PagerCommand::AcceptSearch => self.mode = PagerMode::Navigate,
                _ => {}
            }
        } else {
            let maximum_offset = total_lines.saturating_sub(viewport_height);
            let half_page = (viewport_height / 2).max(1);
            match command {
                PagerCommand::Down => self.offset = self.offset.saturating_add(1),
                PagerCommand::Up => self.offset = self.offset.saturating_sub(1),
                PagerCommand::HalfDown => {
                    self.offset = self.offset.saturating_add(half_page);
                }
                PagerCommand::HalfUp => self.offset = self.offset.saturating_sub(half_page),
                PagerCommand::End => self.offset = maximum_offset,
                PagerCommand::Start => self.offset = 0,
                PagerCommand::Search => self.mode = PagerMode::Search,
                PagerCommand::All => {
                    self.access_filter = AccessFilter::All;
                    self.offset = 0;
                }
                PagerCommand::Hybrid => {
                    self.access_filter = AccessFilter::Hybrid;
                    self.offset = 0;
                }
                PagerCommand::InPerson => {
                    self.access_filter = AccessFilter::InPerson;
                    self.offset = 0;
                }
                PagerCommand::Online => {
                    self.access_filter = AccessFilter::Online;
                    self.offset = 0;
                }
                PagerCommand::Reset => *self = Self::default(),
                _ => {}
            }
            self.offset = self.offset.min(maximum_offset);
        }

        PagerOutcome::Continue
    }

    pub(super) const fn offset(&self) -> usize {
        self.offset
    }

    pub(super) fn query(&self) -> &str {
        &self.query
    }

    pub(super) fn is_searching(&self) -> bool {
        self.mode == PagerMode::Search
    }

    pub(super) const fn access_filter(&self) -> AccessFilter {
        self.access_filter
    }

    fn clamp(&mut self, total_lines: usize, viewport_height: usize) {
        self.offset = self.offset.min(total_lines.saturating_sub(viewport_height));
    }
}

const fn command_is_quit(command: PagerCommand) -> bool {
    matches!(command, PagerCommand::Quit)
}

fn command_for_key(key: KeyEvent, is_searching: bool) -> Option<PagerCommand> {
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(PagerCommand::Quit);
    }
    if is_searching {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('u') {
            return Some(PagerCommand::ClearSearch);
        }
        return match key.code {
            KeyCode::Enter | KeyCode::Esc => Some(PagerCommand::AcceptSearch),
            KeyCode::Backspace => Some(PagerCommand::Backspace),
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                Some(PagerCommand::Character(character))
            }
            _ => None,
        };
    }

    match key.code {
        KeyCode::Down | KeyCode::Char('j') => Some(PagerCommand::Down),
        KeyCode::Up | KeyCode::Char('k') => Some(PagerCommand::Up),
        KeyCode::PageDown | KeyCode::Char('d') => Some(PagerCommand::HalfDown),
        KeyCode::PageUp | KeyCode::Char('u') => Some(PagerCommand::HalfUp),
        KeyCode::Char('G') | KeyCode::End => Some(PagerCommand::End),
        KeyCode::Home | KeyCode::Char('g') => Some(PagerCommand::Start),
        KeyCode::Char('/') => Some(PagerCommand::Search),
        KeyCode::Char('a') => Some(PagerCommand::All),
        KeyCode::Char('h') => Some(PagerCommand::Hybrid),
        KeyCode::Char('p') => Some(PagerCommand::InPerson),
        KeyCode::Char('o') => Some(PagerCommand::Online),
        KeyCode::Char('r') => Some(PagerCommand::Reset),
        KeyCode::Char('q') | KeyCode::Esc => Some(PagerCommand::Quit),
        _ => None,
    }
}

const fn should_page(
    pager_enabled: bool,
    input_is_terminal: bool,
    output_is_terminal: bool,
) -> bool {
    pager_enabled && input_is_terminal && output_is_terminal
}

pub(super) fn display(list: &MeetingList, pager_enabled: bool, theme: Theme) -> io::Result<()> {
    if should_page(
        pager_enabled,
        io::stdin().is_terminal(),
        io::stdout().is_terminal(),
    ) {
        browse(list, theme)
    } else {
        let mut stdout = io::stdout().lock();
        stdout.write_all(list.render_static(theme).as_bytes())
    }
}

fn browse(list: &MeetingList, theme: Theme) -> io::Result<()> {
    let mut terminal = TerminalSession::enter()?;
    let mut state = PagerState::default();

    loop {
        let (columns, rows) = terminal::size()?;
        let width = usize::from(columns);
        let height = usize::from(rows).max(3);
        let body_height = height.saturating_sub(2).max(1);
        let total_lines = list
            .rendered_lines(state.query(), state.access_filter(), width, theme)
            .len();
        let frame = render_frame(list, &mut state, width, height, theme);
        terminal.draw(&frame)?;

        if let Event::Key(key) = event::read()? {
            let Some(command) = command_for_key(key, state.is_searching()) else {
                continue;
            };
            if state.apply(command, total_lines, body_height) == PagerOutcome::Quit {
                return Ok(());
            }
        }
    }
}

struct TerminalSession {
    stdout: io::Stdout,
}

impl TerminalSession {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, Hide) {
            let _ = terminal::disable_raw_mode();
            let _ = execute!(stdout, Show, LeaveAlternateScreen);
            return Err(error);
        }
        Ok(Self { stdout })
    }

    fn draw(&mut self, frame: &str) -> io::Result<()> {
        queue!(
            self.stdout,
            MoveTo(0, 0),
            Clear(ClearType::All),
            Print(frame)
        )?;
        self.stdout.flush()
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(self.stdout, Show, LeaveAlternateScreen);
    }
}

fn render_frame(
    list: &MeetingList,
    state: &mut PagerState,
    width: usize,
    height: usize,
    theme: Theme,
) -> String {
    let body_height = height.saturating_sub(2).max(1);
    let lines = list.rendered_lines(state.query(), state.access_filter(), width, theme);
    state.clamp(lines.len(), body_height);
    let matching_count = list.matching_count(state.query(), state.access_filter());
    let total_count = list.total_count();
    let count = meeting_count_label(
        matching_count,
        total_count,
        !state.query().is_empty() || state.access_filter() != AccessFilter::All,
    );
    let header = theme.heading(&truncate_to_width(
        &format!("alc meetings  {count}  [{}]", state.access_filter().label()),
        width,
    ));
    let footer = if state.is_searching() {
        theme.prompt(&truncate_to_width(
            &format!(
                "Filter: {}█  Ctrl+U clear  Backspace empty exits  Enter/Esc finish",
                state.query()
            ),
            width,
        ))
    } else {
        theme.muted(&truncate_to_width(
            "j/k ↑/↓ move  d/u half  / search  G end  a/h/p/o access  r reset  q quit",
            width,
        ))
    };
    let mut frame = Vec::with_capacity(height.max(3));
    frame.push(header);
    frame.extend(lines.iter().skip(state.offset()).take(body_height).cloned());
    while frame.len() < body_height + 1 {
        frame.push(String::new());
    }
    frame.push(footer);
    frame.join("\r\n")
}

fn meeting_count_label(matching_count: usize, total_count: usize, filter_active: bool) -> String {
    let noun = if total_count == 1 {
        "meeting"
    } else {
        "meetings"
    };
    if filter_active {
        format!("{matching_count} of {total_count} {noun}")
    } else {
        format!("{total_count} {noun}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{meeting::parse_directory, theme::Theme};
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn navigation_commands_move_and_clamp_the_viewport() {
        let mut state = PagerState::default();

        assert_eq!(
            state.apply(PagerCommand::HalfDown, 100, 20),
            PagerOutcome::Continue
        );
        assert_eq!(state.offset(), 10);
        state.apply(PagerCommand::Down, 100, 20);
        assert_eq!(state.offset(), 11);
        state.apply(PagerCommand::Up, 100, 20);
        assert_eq!(state.offset(), 10);
        state.apply(PagerCommand::End, 100, 20);
        assert_eq!(state.offset(), 80);
        state.apply(PagerCommand::HalfUp, 100, 20);
        assert_eq!(state.offset(), 70);
        state.apply(PagerCommand::Start, 100, 20);
        assert_eq!(state.offset(), 0);
        assert_eq!(state.apply(PagerCommand::Quit, 100, 20), PagerOutcome::Quit);
    }

    #[test]
    fn search_mode_filters_live_and_keeps_q_as_search_text() {
        let mut state = PagerState::default();
        state.apply(PagerCommand::End, 100, 20);

        state.apply(PagerCommand::Search, 100, 20);
        assert!(state.is_searching());
        state.apply(PagerCommand::Character('B'), 100, 20);
        state.apply(PagerCommand::Character('q'), 100, 20);
        assert_eq!(state.query(), "Bq");
        assert_eq!(state.offset(), 0);
        state.apply(PagerCommand::Backspace, 100, 20);
        assert_eq!(state.query(), "B");
        state.apply(PagerCommand::AcceptSearch, 100, 20);
        assert!(!state.is_searching());
        assert_eq!(state.query(), "B");
    }

    #[test]
    fn ctrl_u_clears_search_and_empty_backspace_exits_search_mode() {
        let mut state = PagerState::default();
        state.apply(PagerCommand::Search, 100, 20);
        for character in "brook".chars() {
            state.apply(PagerCommand::Character(character), 100, 20);
        }

        state.apply(PagerCommand::ClearSearch, 100, 20);

        assert_eq!(state.query(), "");
        assert!(state.is_searching());
        assert_eq!(state.offset(), 0);

        state.apply(PagerCommand::Backspace, 100, 20);

        assert_eq!(state.query(), "");
        assert!(!state.is_searching());
    }

    #[test]
    fn terminal_keys_map_to_less_like_commands() {
        let key = |code| KeyEvent::new(code, KeyModifiers::NONE);
        let control_key = |code| KeyEvent::new(code, KeyModifiers::CONTROL);

        assert!(matches!(
            command_for_key(key(KeyCode::Char('j')), false),
            Some(PagerCommand::Down)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Down), false),
            Some(PagerCommand::Down)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('k')), false),
            Some(PagerCommand::Up)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Up), false),
            Some(PagerCommand::Up)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('d')), false),
            Some(PagerCommand::HalfDown)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('u')), false),
            Some(PagerCommand::HalfUp)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('G')), false),
            Some(PagerCommand::End)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('/')), false),
            Some(PagerCommand::Search)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('q')), false),
            Some(PagerCommand::Quit)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('a')), false),
            Some(PagerCommand::All)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('h')), false),
            Some(PagerCommand::Hybrid)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('p')), false),
            Some(PagerCommand::InPerson)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('o')), false),
            Some(PagerCommand::Online)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('r')), false),
            Some(PagerCommand::Reset)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('q')), true),
            Some(PagerCommand::Character('q'))
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('h')), true),
            Some(PagerCommand::Character('h'))
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Char('r')), true),
            Some(PagerCommand::Character('r'))
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Backspace), true),
            Some(PagerCommand::Backspace)
        ));
        assert!(matches!(
            command_for_key(control_key(KeyCode::Char('u')), true),
            Some(PagerCommand::ClearSearch)
        ));
        assert!(matches!(
            command_for_key(key(KeyCode::Enter), true),
            Some(PagerCommand::AcceptSearch)
        ));
    }

    #[test]
    fn pager_frame_filters_whole_meetings_and_highlights_matches() {
        let list = fixture_list();
        let mut state = PagerState::default();
        state.apply(PagerCommand::Search, 100, 20);
        for character in "brook".chars() {
            state.apply(PagerCommand::Character(character), 100, 20);
        }

        let frame = render_frame(&list, &mut state, 100, 40, Theme::dark(true));

        assert!(frame.contains("1 of 2 meetings"));
        assert!(frame.contains("Harbor Light"));
        assert!(frame.contains("https://zoom.example.test/harbor"));
        assert!(frame.contains("https://nyintergroup.org/harbor"));
        assert!(!frame.contains("Uptown Noon"));
        assert!(frame.contains("\x1b[1;30;103mBrook\x1b[0m"));
        assert!(frame.contains("Filter: brook"));
    }

    #[test]
    fn access_shortcuts_filter_whole_meetings_and_reset_the_viewport() {
        let list = fixture_list();
        let mut state = PagerState::default();
        state.apply(PagerCommand::End, 100, 20);

        state.apply(PagerCommand::Hybrid, 100, 20);
        assert_eq!(state.access_filter(), AccessFilter::Hybrid);
        assert_eq!(state.offset(), 0);
        let hybrid = render_frame(&list, &mut state, 100, 40, Theme::dark(false));
        assert!(hybrid.contains("Harbor Light"));
        assert!(!hybrid.contains("Uptown Noon"));
        assert!(hybrid.contains("hybrid"));

        state.apply(PagerCommand::Online, 100, 20);
        assert_eq!(state.access_filter(), AccessFilter::Online);
        let online = render_frame(&list, &mut state, 100, 40, Theme::dark(false));
        assert!(online.contains("Harbor Light"));
        assert!(online.contains("Uptown Noon"));

        state.apply(PagerCommand::InPerson, 100, 20);
        assert_eq!(state.access_filter(), AccessFilter::InPerson);
        let in_person = render_frame(&list, &mut state, 100, 40, Theme::dark(false));
        assert!(in_person.contains("Harbor Light"));
        assert!(!in_person.contains("Uptown Noon"));

        state.apply(PagerCommand::All, 100, 20);
        assert_eq!(state.access_filter(), AccessFilter::All);
    }

    #[test]
    fn reset_restores_the_original_view_at_the_top() {
        let mut state = PagerState::default();
        state.apply(PagerCommand::End, 100, 20);
        state.apply(PagerCommand::Hybrid, 100, 20);
        state.apply(PagerCommand::Search, 100, 20);
        for character in "brooklyn".chars() {
            state.apply(PagerCommand::Character(character), 100, 20);
        }
        state.apply(PagerCommand::AcceptSearch, 100, 20);

        state.apply(PagerCommand::Reset, 100, 20);

        assert_eq!(state.offset(), 0);
        assert_eq!(state.query(), "");
        assert_eq!(state.access_filter(), AccessFilter::All);
        assert!(!state.is_searching());
    }

    #[test]
    fn pager_frame_does_not_wrap_past_a_narrow_terminal() {
        let frame = render_frame(
            &fixture_list(),
            &mut PagerState::default(),
            30,
            10,
            Theme::dark(false),
        );

        for line in frame.lines() {
            assert!(
                UnicodeWidthStr::width(line) <= 30,
                "line should fit the terminal: {line}"
            );
        }
    }

    fn fixture_list() -> MeetingList {
        let meetings = parse_directory(
            r#"[
                {
                    "id": 1,
                    "name": "Harbor Light",
                    "slug": "harbor-light",
                    "url": "https://nyintergroup.org/harbor",
                    "day": 0,
                    "time": "09:00",
                    "end_time": "10:00",
                    "types": ["Open", "Discussion"],
                    "conference_url": "https://zoom.example.test/harbor",
                    "formatted_address": "10 Water St, Brooklyn",
                    "regions": ["Brooklyn"],
                    "attendance_option": "hybrid"
                },
                {
                    "id": 2,
                    "name": "Uptown Noon",
                    "slug": "uptown-noon",
                    "url": "https://nyintergroup.org/uptown",
                    "day": 1,
                    "time": "12:00",
                    "end_time": "13:00",
                    "types": ["Open"],
                    "conference_url": "https://zoom.example.test/uptown",
                    "regions": ["Manhattan"],
                    "attendance_option": "online"
                }
            ]"#,
        )
        .expect("fixture should parse");
        super::super::render::find_list(&meetings.iter().collect::<Vec<_>>(), None)
    }

    #[test]
    fn pager_requires_opt_in_and_an_interactive_input_and_output() {
        assert!(should_page(true, true, true));
        assert!(!should_page(false, true, true));
        assert!(!should_page(true, false, true));
        assert!(!should_page(true, true, false));
    }

    #[test]
    fn pager_count_uses_singular_meeting_for_one_result() {
        assert_eq!(meeting_count_label(1, 1, false), "1 meeting");
        assert_eq!(meeting_count_label(1, 20, true), "1 of 20 meetings");
    }
}
