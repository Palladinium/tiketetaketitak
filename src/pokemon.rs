use std::{
    fmt::{self, Display, Formatter},
    rc::Rc,
};

use arrayvec::ArrayVec;

use crate::{ability::Ability, item::Item, pokemove::PokeMove};

#[derive(Debug, Clone)]
pub struct PokemonSpecies {
    pub national_dex_no: u32,
    pub name: String,
    pub forms: Vec<Rc<PokemonForm>>,
}

impl Display for PokemonSpecies {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct PokemonForm {
    pub species: Rc<PokemonSpecies>,
    pub name: Option<String>,

    pub types: Vec<PokeType>,
    pub genders: AllowedGenders,
    pub base_stats: Stats,
}

impl Display for PokemonForm {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(n) = self.name.as_ref() {
            write!(f, "{} - {}", self.species.name, n)
        } else {
            write!(f, "{}", self.species.name)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub special_attack: u32,
    pub special_defense: u32,
    pub speed: u32,
}

#[derive(Debug, Clone)]
pub struct Pokemon {
    pub form: Rc<PokemonForm>,

    pub nickname: Option<String>,

    pub gender: Gender,
    pub moves: ArrayVec<[PokeMove; 4]>,

    pub ev: Stats,
    pub iv: Stats,
    pub ability: Ability,
    pub item: Option<Item>,
}

impl Display for Pokemon {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(n) = self.nickname.as_ref() {
            write!(f, "{} ({})", n, self.form)
        } else {
            write!(f, "{}", self.form)
        }
    }
}

#[derive(Debug, Clone)]
pub enum AllowedGenders {
    MaleOrFemale,
    MaleOnly,
    FemaleOnly,
    NoGender,
}

impl AllowedGenders {
    pub fn as_slice(&self) -> &'static [Gender] {
        match self {
            AllowedGenders::MaleOrFemale => &[Gender::Male, Gender::Female],
            AllowedGenders::MaleOnly => &[Gender::Male],
            AllowedGenders::FemaleOnly => &[Gender::Female],
            AllowedGenders::NoGender => &[Gender::None],
        }
    }
}

#[derive(Debug, Clone)]
pub enum Gender {
    None,
    Male,
    Female,
}

#[derive(Debug, Clone)]
pub enum PokeType {
    Normal,
    Fire,
    Water,
    Electric,
    Grass,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
}

impl PokeType {
    pub fn effectiveness_on(&self, defender: &Self) -> TypeEffectiveness {
        use PokeType::*;
        use TypeEffectiveness::*;

        match self {
            Normal => match defender {
                Ghost => NoEffect,
                Rock | Steel => NotVeryEffective,
                _ => Regular,
            },
            Fire => match defender {
                Fire | Water | Rock | Dragon => NotVeryEffective,
                Grass | Ice | Bug | Steel => SuperEffective,
                _ => Regular,
            },
            Water => match defender {
                Water | Grass | Dragon => NotVeryEffective,
                Fire | Ground | Rock => SuperEffective,
                _ => Regular,
            },
            Electric => match defender {
                Ground => NoEffect,
                Electric | Grass | Dragon => NotVeryEffective,
                Water | Flying => SuperEffective,
                _ => Regular,
            },
            Grass => match defender {
                Fire | Grass | Poison | Flying | Bug | Dragon | Steel => NotVeryEffective,
                Water | Ground | Rock => SuperEffective,
                _ => Regular,
            },
            Ice => match defender {
                Fire | Water | Ice | Steel => NotVeryEffective,
                Grass | Ground | Flying | Dragon => SuperEffective,
                _ => Regular,
            },
            Fighting => match defender {
                Poison | Flying | Psychic | Bug | Fairy => NotVeryEffective,
                Normal | Ice | Rock | Dark | Steel => SuperEffective,
                Ghost => NoEffect,
                _ => Regular,
            },
            Poison => match defender {
                Steel => NoEffect,
                Poison | Ground | Rock | Ghost => NotVeryEffective,
                Grass | Fairy => SuperEffective,
                _ => Regular,
            },
            Ground => match defender {
                Flying => NoEffect,
                Grass | Bug => NotVeryEffective,
                Fire | Electric | Poison | Rock | Steel => SuperEffective,
                _ => Regular,
            },
            Flying => match defender {
                Electric | Rock | Steel => NotVeryEffective,
                Grass | Fighting | Bug => SuperEffective,
                _ => Regular,
            },
            Psychic => match defender {
                Dark => NoEffect,
                Psychic | Steel => NotVeryEffective,
                Fighting | Poison => SuperEffective,
                _ => Regular,
            },
            Bug => match defender {
                Fire | Fighting | Poison | Flying | Ghost | Steel | Fairy => NotVeryEffective,
                Grass | Psychic | Dark => SuperEffective,
                _ => Regular,
            },
            Rock => match defender {
                Fighting | Ground | Steel => NotVeryEffective,
                Fire | Ice | Flying | Bug => SuperEffective,
                _ => Regular,
            },
            Ghost => match defender {
                Normal => NoEffect,
                Dark => NotVeryEffective,
                Psychic | Ghost => SuperEffective,
                _ => Regular,
            },
            Dragon => match defender {
                Fairy => NoEffect,
                Steel => NotVeryEffective,
                Dragon => SuperEffective,
                _ => Regular,
            },
            Dark => match defender {
                Fighting | Dragon | Fairy => NotVeryEffective,
                Psychic | Ghost => SuperEffective,
                _ => Regular,
            },
            Steel => match defender {
                Fire | Water | Electric | Steel => NotVeryEffective,
                Ice | Rock | Fairy => SuperEffective,
                _ => Regular,
            },
            Fairy => match defender {
                Fire | Poison | Steel => NotVeryEffective,
                Fighting | Dragon | Dark => SuperEffective,
                _ => Regular,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeEffectiveness {
    NoEffect,
    NotVeryEffective,
    Regular,
    SuperEffective,
}

pub type Team = ArrayVec<[Pokemon; 6]>;
