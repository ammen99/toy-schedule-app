use std::rc::Rc;

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
    button: iced::button::State,
}

impl Activity {
    fn new() -> Activity {
        Activity {
            name: String::from(""),
            url: String::from(""),
            class_type: ClassType::Lecture,
            button: iced::button::State::default(),
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
    name: iced::text_input::State,
    url: iced::text_input::State,
    class_type: Option<ClassType>,

    activity: Activity,
}

impl NewActivityInput {
    fn new() -> NewActivityInput {
        NewActivityInput {
            name: iced::text_input::State::default(),
            url: iced::text_input::State::default(),
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
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum ScheduleMessage {
    // A new activity has been requested
    NewActivityRequest,

    // The type of the new activity has been selected
    NewActivityTypeSelected(ClassType),

    // The properties of the new activity have been changed
    NewActivityTextChanged,
}

impl iced::Sandbox for Schedule {
    type Message = ScheduleMessage;

    fn new() -> Schedule {
        return Schedule {
            activities: vec![],
            time_plan: TimePlan::default(),
            new_activity: None,
            new_activity_btn: iced::button::State::default(),
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
            ScheduleMessage::NewActivityTextChanged => {
            }
        }
    }

    fn view(&mut self) -> iced::Element<ScheduleMessage> {
        let mut content = iced::Column::new()
            .padding(20).align_items(iced::Align::Center);

        content = self.activities.iter().fold(content, |content, activity| {
            content.push(iced::Text::new((*activity).name.clone()))
        });

        match &self.new_activity {
            Some(_) => {
                content = content.push(self.new_activity_layout());
            }
            None => {
                let btn = iced::Button::new(&mut self.new_activity_btn,
                                            iced::Text::new("Add new activity"))
                    .on_press(ScheduleMessage::NewActivityRequest);

                content = content.push(btn);
            }
        };

        content.into()
    }
}

impl Schedule {
    fn new_activity_layout(&mut self) -> iced::Column<ScheduleMessage> {
        if let Some(new_activity) = &mut self.new_activity {
            let new_label = |state, label| {
                iced::TextInput::new(
                    state,
                    &".".repeat(20).as_str(),
                    label,
                    |_| ScheduleMessage::NewActivityTextChanged)
            };

            let new_radio = |selected, value, label| {
                iced::Radio::new(
                    value, label, selected,
                    ScheduleMessage::NewActivityTypeSelected)
            };

            iced::Column::new()
                .spacing(20)
                .align_items(iced::Align::Start)
                .push(new_label(&mut new_activity.name, "Activity name"))
                .push(new_label(&mut new_activity.url, "Activity URL"))
                .push(new_radio(new_activity.class_type, ClassType::Lecture, "Lecture"))
                .push(new_radio(new_activity.class_type, ClassType::ProblemClass, "Problem Class"))
                .push(new_radio(new_activity.class_type, ClassType::Tutorial, "Tutorial"))
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
