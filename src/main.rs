mod style;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Activity {
    name: String,
    url: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ActivityPickListItem {
    index: usize,
    label: String,
}

impl ToString for ActivityPickListItem {
    fn to_string(&self) -> String {
        self.label.clone()
    }
}

#[derive(Default)]
struct ScheduledActivity {
    activity: Option<ActivityPickListItem>,
    pick_state: iced::pick_list::State<ActivityPickListItem>,
    link_state: iced::button::State,
}

type DayPlan = [ScheduledActivity; 6];
type TimePlan = [DayPlan; 5];

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum NewActivityTextInputs {
    Name,
    URL,
}

impl NewActivityTextInputs {
    fn get_placeholder(&self) -> String {
        match *self {
            NewActivityTextInputs::Name => { String::from("Enter activity name") }
            NewActivityTextInputs::URL => { String::from("Enter activity URL") }
        }
    }
}

struct ActivityCreateParams {
    name_state: iced::text_input::State,
    name: String,
    url_state: iced::text_input::State,
    url: String,

    new_activity_submit_btn: iced::button::State,
}

struct ActivitiesArea {
    // New activities input
    new_activity: ActivityCreateParams,
    adding_activity: bool,

    // Buttons
    new_activity_btn: iced::button::State,
}

impl ActivitiesArea {
    fn new() -> ActivitiesArea {
        ActivitiesArea {
            new_activity: ActivityCreateParams {
                name_state: iced::text_input::State::default(),
                name: String::from(""),
                url_state: iced::text_input::State::default(),
                url: String::from(""),
                new_activity_submit_btn: iced::button::State::default(),
            },
            adding_activity: false,

            new_activity_btn: iced::button::State::default(),
        }
    }
}

struct Schedule {
    activity_area: ActivitiesArea,
    activities: Vec<Activity>,
    time_plan : TimePlan,
    theme: style::Theme,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ScheduleMessage {
    // A new activity has been requested
    NewActivityRequest,

    // New activity text updated
    NewActivityTextChanged(NewActivityTextInputs, String),

    // New activity should be created
    NewActivitySubmitted,

    // Activity chosen (day, block, idx)
    ActivityChosen(usize, usize, Option<usize>),

    // Launch meeting
    LaunchMeeting(String),
}

static CAPTIONS: &'static [&'static str] =
&["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"];

fn time_plan_layout<'a>(plan: &'a mut TimePlan, activities: &mut Vec<Activity>, theme: style::Theme)
        -> iced::Element<'a, ScheduleMessage> {
    let mut content = iced::Row::<ScheduleMessage>::new()
        .push(iced::Rule::vertical(10));

    let pick_list_items: Vec<ActivityPickListItem> =
        activities.iter().enumerate().map(|(i, activity)| {
            ActivityPickListItem {index: i, label: activity.name.clone()}
        }).collect();

    for (day_idx, day) in plan.iter_mut().enumerate() {
        let mut clock_begin = 8; // 08:00
        let mut day_column =
            iced::Column::<ScheduleMessage>::new()
          //  .push(iced::Space::with_height(iced::Length::Units(50)))
            .push(iced::Text::new(CAPTIONS[day_idx].clone()))
            .push(iced::Rule::horizontal(30));

        for (block_idx, block) in day.iter_mut().enumerate() {
            let pick_list = iced::pick_list::PickList::new(
                &mut block.pick_state,
                pick_list_items.clone(),
                block.activity.clone(),
                move |sel| { ScheduleMessage::ActivityChosen(day_idx, block_idx, Some(sel.index)) })
                .style(theme);

            let mut url_btn = iced::Button::new(&mut block.link_state, iced::Text::new("Meeting"))
                .style(theme);
            if let Some(activity) = &mut block.activity {
                url_btn = url_btn
                    .on_press(ScheduleMessage::LaunchMeeting(
                            activities[activity.index].url.clone()));
            }

            let block_column = iced::Column::new()
                .push(iced::Text::new(format!("{:0>2}:00", clock_begin))
                      .horizontal_alignment(iced::HorizontalAlignment::Left)
                      .size(16)
                      .color(iced::Color::from_rgb(0.5, 0.5, 0.5)))
                .push(iced::Space::with_height(iced::Length::Units(10)))
                .push(iced::Container::new(
                        iced::Column::new()
                        .push(pick_list.width(iced::Length::Fill))
                        .push(iced::Space::with_height(iced::Length::Units(20)))
                        .push(iced::Container::new(url_btn)
                              .align_x(iced::Align::Center)
                              .width(iced::Length::Fill)))
                    .style(theme)
                    .width(iced::Length::Fill)
                    .align_x(iced::Align::Center))
                .push(iced::Rule::horizontal(30));

            day_column = day_column.push(block_column);
            clock_begin += 2; // + 02:00
        }

        content = content
            .push(day_column.max_width(150))
            .push(iced::Rule::vertical(10));
    }

    content.max_height(900).into()
}

impl iced::Sandbox for Schedule {
    type Message = ScheduleMessage;

    fn new() -> Schedule {
        return Schedule {
            activity_area: ActivitiesArea::new(),
            time_plan: TimePlan::default(),
            theme: style::Theme::Dark,
            activities: vec![],
        }
    }

    fn title(&self) -> String {
        return String::from("Class scheduler");
    }

    fn update(&mut self, message: ScheduleMessage) {
        let new_activity = &mut self.activity_area.new_activity;
        match message {
            ScheduleMessage::NewActivityRequest => {
                assert_eq!(self.activity_area.adding_activity, false);
                self.activity_area.adding_activity = true;
            }

            ScheduleMessage::NewActivityTextChanged(input, value) => {
                assert_eq!(self.activity_area.adding_activity, true);
                match input {
                    NewActivityTextInputs::Name => {
                        new_activity.name = value;
                    }
                    NewActivityTextInputs::URL => {
                        new_activity.url = value;
                    }
                }
            }

            ScheduleMessage::NewActivitySubmitted => {
                assert_eq!(self.activity_area.adding_activity, true);
                self.activities.push(Activity {
                    name: new_activity.name.clone(),
                    url: new_activity.url.clone(),
                });
                self.activity_area.adding_activity = false;
            }

            ScheduleMessage::ActivityChosen(day, block, idx) => {
                self.time_plan[day][block].activity = Some(ActivityPickListItem {
                    index: idx.unwrap(),
                    label: self.activities[idx.unwrap()].name.clone(),
                });
            }

            ScheduleMessage::LaunchMeeting(url) => {
                open::with(url.clone(), "google-chrome-unstable").ok();
            }
        }
    }

    fn view(&mut self) -> iced::Element<ScheduleMessage> {
        let theme = self.theme;

        let activities = self.activity_area.layout(theme, &mut self.activities);
        let table = time_plan_layout(&mut self.time_plan, &mut self.activities, theme);

        let content = iced::Row::new()
            .padding(20)
            .push(table)
            .push(activities);

        iced::Container::new(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(theme)
            .into()
    }
}

impl ActivitiesArea {
    fn layout<'a>(&'a mut self,
              theme: style::Theme, activities: &mut Vec<Activity>) -> iced::Column<'a, ScheduleMessage> {
        let mut content = iced::Column::new()
            .padding(20).align_items(iced::Align::Center);

        content = activities.iter().fold(content, |content, activity| {
            content.push(iced::Text::new((*activity).name.clone()))
        });

        if self.adding_activity {
            content = content.push(self.new_activity.layout(theme));
        } else {
            let btn = iced::Button::new(&mut self.new_activity_btn,
                                        iced::Text::new("Add new activity"))
                .on_press(ScheduleMessage::NewActivityRequest)
                .style(theme);

            content = content.push(btn);
        }

        content
    }
}

impl ActivityCreateParams {
    fn layout(&mut self, theme: style::Theme) -> iced::Column<ScheduleMessage> {
        let new_label = |state, msg: NewActivityTextInputs, value| {
            iced::TextInput::new(
                state,
                &msg.get_placeholder().as_str(),
                value,
                move |new_value| ScheduleMessage::NewActivityTextChanged(msg, new_value))
                .style(theme)
        };

        iced::Column::new()
            .spacing(20)
            .align_items(iced::Align::Start)
            .push(new_label(&mut self.name_state, NewActivityTextInputs::Name, &self.name))
            .push(new_label(&mut self.url_state, NewActivityTextInputs::URL, &self.url))
            .push(
                iced::Button::new(&mut self.new_activity_submit_btn,
                                  iced::Text::new("Create activity"))
                .on_press(ScheduleMessage::NewActivitySubmitted)
                .style(theme))
    }
}

pub fn main() {
    use iced::Sandbox;

    let mut stgs = iced::Settings::default();
    stgs.window.size = (1300, 930);
    match Schedule::run(stgs) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("Failed to run program");
        }
    }
}
