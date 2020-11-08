use std::rc::Rc;
mod style;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum ClassType {
    Lecture,
    ProblemClass,
    Tutorial
}

struct Activity {
    name: String,
    url: String,
    class_type: ClassType,
    remove: iced::button::State,
}

impl Activity {
    fn new() -> Activity {
        Activity {
            name: String::from(""),
            url: String::from(""),
            class_type: ClassType::Lecture,
            remove: iced::button::State::default(),
        }
    }
}

struct ScheduledClass {
    name: String,
    activity: Rc<Activity>,
}

type DayPlan = [Option<ScheduledClass>; 6];
type TimePlan = [DayPlan; 5];

struct NewActivityInput {
    name_state: iced::text_input::State,
    name: String,
    url_state: iced::text_input::State,
    url: String,
    class_type: Option<ClassType>,

    activity: Activity,
}

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

impl NewActivityInput {
    fn new() -> NewActivityInput {
        NewActivityInput {
            name_state: iced::text_input::State::default(),
            name: String::from(""),
            url_state: iced::text_input::State::default(),
            url: String::from(""),
            class_type: None,
            activity: Activity::new(),
        }
    }
}

struct Schedule {
    activities : Vec<Rc<Activity>>,
    time_plan : TimePlan,

    // GUI elements
    new_activity: Option<NewActivityInput>,
    new_activity_btn: iced::button::State,
    new_activity_submit_btn: iced::button::State,

    theme: style::Theme,
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum ScheduleMessage {
    // A new activity has been requested
    NewActivityRequest,

    // The type of the new activity has been selected
    NewActivityTypeSelected(ClassType),

    // New activity text updated
    NewActivityTextChanged(NewActivityTextInputs, String),

    // New activity should be created
    NewActivitySubmitted,
}

impl iced::Sandbox for Schedule {
    type Message = ScheduleMessage;

    fn new() -> Schedule {
        return Schedule {
            activities: vec![],
            time_plan: TimePlan::default(),
            new_activity: None,
            new_activity_btn: iced::button::State::default(),
            new_activity_submit_btn: iced::button::State::default(),
            theme: style::Theme::Dark,
        }
    }

    fn title(&self) -> String {
        return String::from("Class scheduler");
    }

    fn update(&mut self, message: ScheduleMessage) {
        match message {
            ScheduleMessage::NewActivityRequest => {
                self.new_activity = Some(NewActivityInput::new());
            }

            ScheduleMessage::NewActivityTypeSelected(activity_type) => {
                if let Some(activity) = &mut self.new_activity {
                    activity.class_type = Some(activity_type);
                }
            }

            ScheduleMessage::NewActivityTextChanged(input, value) => {
                if let Some(activity) = &mut self.new_activity {
                    match input {
                        NewActivityTextInputs::Name => {
                            activity.name = value;
                        }
                        NewActivityTextInputs::URL => {
                            activity.url = value;
                        }
                    }
                }
            }

            ScheduleMessage::NewActivitySubmitted => {
                if let Some(activity) = &mut self.new_activity {
                    self.activities.push(Rc::new(Activity {
                        name: activity.name.clone(),
                        url: activity.url.clone(),
                        class_type: activity.class_type.unwrap_or(ClassType::Lecture),
                        remove: iced::button::State::default(),
                    }));
                    self.new_activity = None;
                } else {
                    panic!("Application bug");
                }
            }
        }
    }

    fn view(&mut self) -> iced::Element<ScheduleMessage> {
        let mut content = iced::Column::new()
            .padding(20).align_items(iced::Align::Center);

        content = self.activities.iter().fold(content, |content, activity| {
            content.push(iced::Text::new((*activity).name.clone()))
        });

        let theme = self.theme;
        match &self.new_activity {
            Some(_) => {
                content = content.push(self.new_activity_layout());
            }
            None => {
                let btn = iced::Button::new(&mut self.new_activity_btn,
                                            iced::Text::new("Add new activity"))
                    .on_press(ScheduleMessage::NewActivityRequest)
                    .style(theme);

                content = content.push(btn);
            }
        };

        iced::Container::new(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(theme)
            .into()
    }
}

impl Schedule {
    fn new_activity_layout(&mut self) -> iced::Column<ScheduleMessage> {
        if let Some(new_activity) = &mut self.new_activity {
            let theme = self.theme;
            let new_label = |state, msg: NewActivityTextInputs, value| {
                iced::TextInput::new(
                    state,
                    &msg.get_placeholder().as_str(),
                    value,
                    move |new_value| ScheduleMessage::NewActivityTextChanged(msg, new_value))
                    .style(theme)
            };

            let new_radio = |selected, value, label| {
                iced::Radio::new(value, label, selected,
                    ScheduleMessage::NewActivityTypeSelected)
                    .style(theme)
            };

            iced::Column::new()
                .spacing(20)
                .align_items(iced::Align::Start)
                .push(new_label(&mut new_activity.name_state, NewActivityTextInputs::Name, &new_activity.name))
                .push(new_label(&mut new_activity.url_state, NewActivityTextInputs::URL, &new_activity.url))
                .push(new_radio(new_activity.class_type, ClassType::Lecture, "Lecture"))
                .push(new_radio(new_activity.class_type, ClassType::ProblemClass, "Problem Class"))
                .push(new_radio(new_activity.class_type, ClassType::Tutorial, "Tutorial"))
                .push(
                    iced::Button::new(&mut self.new_activity_submit_btn,
                                      iced::Text::new("Create activity"))
                    .on_press(ScheduleMessage::NewActivitySubmitted)
                    .style(self.theme))
        } else {
            panic!("Should not happen!!!");
        }
    }
}

pub fn main() {
    use iced::Sandbox;
    match Schedule::run(iced::Settings::default()) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("Failed to run program");
        }
    }
}
