use strum_macros::{EnumCount, EnumIter};

#[repr(u64)]
#[derive(Debug, Copy, Clone, EnumIter, EnumCount)]
pub enum Genres {
    PointAndClick = 2,
    Fighting = 4,
    Shooter = 5,
    Music = 7,
    Platform = 8,
    Puzzle = 9,
    Racing = 10,
    RTS = 11,
    RPG = 12,
    Simulator = 13,
    Sport = 14,
    Strategy = 15,
    TBS = 16,
    Tactical = 24,
    HackNSlash = 25,
    Trivia = 26,
    Pinball = 30,
    Adventure = 31,
    Indie = 32,
    Arcade = 33,
    VisualNovel = 34,
    CardAndBoardGame = 35,
    MOBA = 36,
}

#[repr(u64)]
#[derive(Debug, Copy, Clone, EnumIter, EnumCount)]
pub enum Themes {
    Action = 1,
    Fantasy = 17,
    SciFi = 18,
    Horror = 19,
    Thriller = 20,
    Survival = 21,
    Historical = 22,
    Stealth = 23,
    Comedy = 27,
    Business = 28,
    Drama = 31,
    NonFiction = 32,
    Sandbox = 33,
    Educational = 34,
    Kids = 35,
    OpenWorld = 38,
    Warfare = 39,
    Party = 40,
    FourX = 41,
    Erotic = 42,
    Mystery = 43,
    Romance = 44,
}

#[repr(u64)]
#[derive(Debug, Copy, Clone, EnumIter, EnumCount)]
pub enum PlayerPerspective {
    FirstPerson = 1,
    ThirdPerson = 2,
    Isometric = 3,
    SideView = 4,
    Text = 5,
    Auditory = 6,
    VR = 7,
}
