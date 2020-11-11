extern crate savefile;

#[macro_use]
extern crate savefile_derive;

use std::fs;

mod style;


#[derive(Debug, Clone, Eq, PartialEq, Default, Savefile)]
struct Activity {
    name: String,
    url: String,
    id: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Savefile)]
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
    new_activity_cancel_btn: iced::button::State,
}

struct ActivitiesArea {
    // New activities input
    new_activity: ActivityCreateParams,
    editing_activity: Option<usize>,

    // Buttons
    new_activity_btn: iced::button::State,
    activities_erase_btn: Vec<iced::button::State>,
    activities_edit_btn: Vec<iced::button::State>,
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
                new_activity_cancel_btn: iced::button::State::default(),
            },

            editing_activity: None,

            new_activity_btn: iced::button::State::default(),
            activities_erase_btn: vec![],
            activities_edit_btn: vec![],
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

    // Edit the activity with given index
    EditActivityRequest(usize),

    // Cancel editing the activity
    CancelEditRequest,

    // Remove activity (idx)
    RemoveActivity(usize),

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
        .push(iced::Rule::vertical(10).style(theme));

    let pick_list_items: Vec<ActivityPickListItem> =
        activities.iter().map(|activity| {
            ActivityPickListItem {index: activity.id, label: activity.name.clone()}
        }).collect();

    for (day_idx, day) in plan.iter_mut().enumerate() {
        let mut clock_begin = 8; // 08:00
        let mut day_column =
            iced::Column::<ScheduleMessage>::new()
            .push(iced::Rule::horizontal(0).style(theme))
            .push(iced::Space::with_height(iced::Length::Units(15)))
            .push(iced::Text::new(CAPTIONS[day_idx].clone()))
            .push(iced::Rule::horizontal(30).style(theme));

        let length = day.len();

        for (block_idx, block) in day.iter_mut().enumerate() {
            let pick_list = iced::pick_list::PickList::new(
                &mut block.pick_state,
                pick_list_items.clone(),
                block.activity.clone(),
                move |sel| { ScheduleMessage::ActivityChosen(day_idx, block_idx, Some(sel.index)) })
                .style(theme);

            let mut url_btn = iced::Button::new(&mut block.link_state, iced::Text::new("Meeting"));
            if let Some(activity) = &mut block.activity {
                let url = find_activity(activities, Some(activity.index))
                    .map(|a| a.url.clone()).unwrap();

                url_btn = url_btn
                    .on_press(ScheduleMessage::LaunchMeeting(url))
                    .style(theme);
            } else {
                url_btn = url_btn
                    .style(style::InactiveButton);
            }

            let mut block_column = iced::Column::new()
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
                    .align_x(iced::Align::Center));

            if block_idx != length - 1 {
                block_column = block_column
                    .push(iced::Rule::horizontal(30).style(theme));
            } else {
                block_column = block_column
                    .push(iced::Space::with_height(iced::Length::Units(15)))
                    .push(iced::Rule::horizontal(0).style(theme));
            }

            day_column = day_column.push(block_column);
            clock_begin += 2; // + 02:00
        }

        content = content
            .push(day_column.max_width(150))
            .push(iced::Rule::vertical(10).style(theme))
    }

    content
        .max_height(866)
        .into()
}

#[derive(Savefile, Default)]
struct PersistentData {
    activities: Vec<Activity>,
    plan: [[Option<ActivityPickListItem>; 6]; 5],
}

fn get_cfg_file() -> String {
    match std::env::var("HOME") {
        Ok(path) => {
            path + "/.config/plan"
        }
        Err(_) => {
            panic!("Failed to open home directory!");
        }
    }
}

impl Drop for Schedule {
    fn drop(&mut self) {
        let mut data = PersistentData::default();
        data.activities = self.activities.clone();
        for (day_idx, day) in self.time_plan.iter_mut().enumerate() {
            for (block_idx, block) in day.iter_mut().enumerate() {
                data.plan[day_idx][block_idx] = block.activity.clone();
            }
        }

        savefile::save_file(get_cfg_file().as_str(), 1, &data).unwrap();
    }
}

fn find_activity(activities: &mut Vec<Activity>, activity_id: Option<usize>) -> Option<&mut Activity> {
    activities.iter_mut()
        .filter(|activity| { Some(activity.id) == activity_id })
        .nth(0)
}

impl iced::Sandbox for Schedule {
    type Message = ScheduleMessage;

    fn new() -> Schedule {
        let mut instance = Schedule {
            activity_area: ActivitiesArea::new(),
            time_plan: TimePlan::default(),
            theme: style::Theme::Dark,
            activities: vec![],
        };

        if fs::metadata(get_cfg_file()).is_ok() {
            let data = (savefile::load_file(get_cfg_file().as_str(), 1)
                as Result<PersistentData, _>).unwrap();

            instance.activities = data.activities;
            for (day_idx, day) in instance.time_plan.iter_mut().enumerate() {
                for (block_idx, block) in day.iter_mut().enumerate() {
                    block.activity = data.plan[day_idx][block_idx].clone();
                }
            }
        }

        instance
    }

    fn title(&self) -> String {
        return String::from("Class scheduler");
    }

    fn update(&mut self, message: ScheduleMessage) {
        let new_activity = &mut self.activity_area.new_activity;
        match message {
            ScheduleMessage::NewActivityRequest => {
                assert_eq!(self.activity_area.editing_activity, None);
                self.activities.push(Activity::default());

                let mut taken = self.activities.iter()
                    .map(|activity| {activity.id})
                    .collect::<Vec<usize>>();
                taken.sort();
                let mex = taken.iter()
                    .enumerate()
                    .filter_map(|(idx, id)| {
                        if *id == idx { None }
                        else { Some(idx) } })
                    .nth(0).unwrap_or(self.activities.len());

                self.activities.last_mut().unwrap().id = mex;
                self.activity_area.start_edit(self.activities.last().unwrap());
            }

            ScheduleMessage::NewActivityTextChanged(input, value) => {
                assert_ne!(self.activity_area.editing_activity, None);
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
                assert_ne!(self.activity_area.editing_activity, None);

                let activity = find_activity(&mut self.activities,
                                             self.activity_area.editing_activity).unwrap();

                activity.name = new_activity.name.clone();
                activity.url = new_activity.url.clone();
                self.activity_area.editing_activity = None;
            }

            ScheduleMessage::ActivityChosen(day, block, idx) => {
                if let Some(activity) = find_activity(&mut self.activities, idx) {
                    self.time_plan[day][block].activity = Some(ActivityPickListItem {
                        index: idx.unwrap(),
                        label: activity.name.clone(),
                    });
                } else {
                    self.time_plan[day][block].activity = None;
                }
            }

            ScheduleMessage::LaunchMeeting(url) => {
                open::with(url.clone(), "google-chrome-unstable").ok();
            }

            ScheduleMessage::EditActivityRequest(idx) => {
                self.activity_area.start_edit(&self.activities[idx]);
            }

            ScheduleMessage::CancelEditRequest => {
                self.activity_area.editing_activity = None;
            }

            ScheduleMessage::RemoveActivity(remove_idx) => {
                for day in self.time_plan.iter_mut() {
                    for block in day.iter_mut() {
                        if block.activity.as_ref().map(|item| {item.index}) == Some(remove_idx) {
                            block.activity = None;
                        }
                    }
                }

                self.activities.retain(|activity| { activity.id != remove_idx });
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

        self.activities_erase_btn.resize(activities.len(), iced::button::State::new());
        self.activities_edit_btn.resize(activities.len(), iced::button::State::new());

        let btns = self.activities_erase_btn.iter_mut()
            .zip(self.activities_edit_btn.iter_mut());

        content = activities.iter()
            .zip(btns)
            .fold(content, |content, (activity, (erase, edit))| {
                content
                    .push(iced::Row::new()
                          .push(iced::Button::new(erase, iced::Text::new("X")
                                                  .horizontal_alignment(iced::HorizontalAlignment::Center))
                                .on_press(ScheduleMessage::RemoveActivity(activity.id))
                                .style(style::Theme::Light)
                                .width(iced::Length::Units(30))
                                .height(iced::Length::Units(30)))
                          .push(iced::Space::with_width(iced::Length::Units(10)))
                          .push(iced::Button::new(edit, iced::Text::new("E")
                                                  .horizontal_alignment(iced::HorizontalAlignment::Center))
                                .on_press(ScheduleMessage::EditActivityRequest(activity.id))
                                .style(style::EditButton)
                                .width(iced::Length::Units(30))
                                .height(iced::Length::Units(30)))
                          .push(iced::Space::with_width(iced::Length::Units(10)))
                          .push(iced::Text::new((*activity).name.clone())
                                .horizontal_alignment(iced::HorizontalAlignment::Left)
                                .vertical_alignment(iced::VerticalAlignment::Center)
                                .height(iced::Length::Fill))
                          .width(iced::Length::Units(400))
                          .height(iced::Length::Units(30)))
                    .push(iced::Space::with_height(iced::Length::Units(5)))
            });

        content = content.push(iced::Space::with_height(iced::Length::Units(10)));

        if self.editing_activity != None {
            content = content.push(self.new_activity.layout(theme));
        } else {
            let btn = iced::Button::new(&mut self.new_activity_btn,
                                        iced::Text::new("Add new activity"))
                .on_press(ScheduleMessage::NewActivityRequest)
                .style(theme);

            content = content.push(btn);
        }

        content.align_items(iced::Align::Start)
    }

    fn start_edit(&mut self, activity: &Activity) {
        self.editing_activity = Some(activity.id);
        self.new_activity.name = activity.name.clone();
        self.new_activity.url = activity.url.clone();
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
            .push(iced::Row::new()
                  .push(iced::Button::new(&mut self.new_activity_submit_btn,
                                          iced::Text::new("Submit"))
                        .on_press(ScheduleMessage::NewActivitySubmitted)
                        .style(theme))
                  .push(iced::Space::with_width(iced::Length::Units(10)))
                  .push(iced::Button::new(&mut self.new_activity_cancel_btn,
                                          iced::Text::new("Cancel"))
                        .on_press(ScheduleMessage::CancelEditRequest)
                        .style(theme)))
    }
}

pub fn main() {
    use iced::Sandbox;

    let mut stgs = iced::Settings::default();
    stgs.window.size = (1300, 906);
    match Schedule::run(stgs) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("Failed to run program");
        }
    }
}
