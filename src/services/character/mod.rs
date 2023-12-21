use self::{class::ClassDefinitions, levels::LevelTables, skill::SkillDefinitions};

pub mod class;
pub mod levels;
pub mod skill;

pub struct CharacterService {
    pub skills: SkillDefinitions,
    pub classes: ClassDefinitions,
    pub level_tables: LevelTables,
}

impl CharacterService {
    pub fn new() -> anyhow::Result<Self> {
        let classes = ClassDefinitions::new()?;
        let skills: SkillDefinitions = SkillDefinitions::new()?;
        let level_tables: LevelTables = LevelTables::new()?;

        Ok(Self {
            classes,
            skills,
            level_tables,
        })
    }
}
