// Character groups for detailed stats
// Generated from tohosort's dataset: 2021-05-09

use serde::{Serialize, Deserialize};
use std::str::FromStr;

// Tell the compiler to stop complaining
#[allow(non_camel_case_types)]

// Group by the work they appeared in
// taken from tohosort

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum Tags {
	book,
	HRtP,
	SoEW,
	PoDD,
	LLS,
	MS,
	EoSD,
	PCB,
	IaMP,
	IN,
	PoFV,
	StB,
	MoF,
	SWR,
	SA,
	UFO,
	soku,
	DS,
	GFW,
	TD,
	HM,
	DDC,
	ISC,
	ULiL,
	LoLK,
	AoCF,
	HSiFS,
	VD,
	WBaWC,
	UM,
	// Stages
	st1,
	st2,
	st3,
	st4,
	st5,
	st6,
	ex,
}

impl Tags {
	// Names of groups (touhou works and stages)
	fn name(&self) -> &'static str {
		match *self {
			Tags::book	=> "Books and CDs",
			Tags::HRtP	=> "The Highly Responsive to Prayers",
			Tags::SoEW	=> "The Story of Eastern Wonderland",
			Tags::PoDD	=> "Phantasmagoria of Dim.Dream",
			Tags::LLS	=> "Lotus Land Story",
			Tags::MS	=> "Mystic Square",
			Tags::EoSD	=> "Embodiment of Scarlet Devil",
			Tags::PCB	=> "Perfect Cherry Blossom",
			Tags::IaMP	=> "Immaterial and Missing Power",
			Tags::IN	=> "Imperishable Night",
			Tags::PoFV	=> "Phantasmagoria of Flower View",
			Tags::StB	=> "Shoot the Bullet",
			Tags::MoF	=> "Mountain of Faith",
			Tags::SWR	=> "Scarlet Weather Rhapsody",
			Tags::SA	=> "Subterranean Animism",
			Tags::UFO	=> "Undefined Fantastic Object",
			Tags::soku	=> "Touhou Hisoutensoku",
			Tags::DS	=> "Double Spoiler",
			Tags::GFW	=> "Great Fairy Wars",
			Tags::TD	=> "Ten Desires",
			Tags::HM	=> "Hopeless Masquerade",
			Tags::DDC	=> "Double Dealing Character",
			Tags::ISC	=> "Impossible Spell Card",
			Tags::ULiL	=> "Urban Legend in Limbo",
			Tags::LoLK	=> "Legacy of Lunatic Kingdom",
			Tags::AoCF	=> "Antinomy of Common Flowers",
			Tags::HSiFS	=> "Hidden Star in Four Seasons",
			Tags::VD	=> "Violet Detector",
			Tags::WBaWC	=> "Wily Beast and Weakest Creature",
			Tags::UM	=> "Unconnected Marketeers",
			// Stages
			Tags::st1	=> "Stage 1",
			Tags::st2	=> "Stage 2",
			Tags::st3	=> "Stage 3",
			Tags::st4	=> "Stage 4",
			Tags::st5	=> "Stage 5/Penultimate",
			Tags::st6	=> "Stage 6/Final",
			Tags::ex	=> "Stage EX/Phantasm",
		}
	}
	// Touhou game titles.
	fn exname(&self) -> &'static str {
		match *self {
			Tags::book	=> "",
			Tags::HRtP	=> "01 - Reiiden",
			Tags::SoEW	=> "02 - Fuumaroku",
			Tags::PoDD	=> "03 - Yumejikuu",
			Tags::LLS	=> "04 - Gensoukyou",
			Tags::MS	=> "05 - Kaikidan",
			Tags::EoSD	=> "06 - Koumakan",
			Tags::PCB	=> "07 - Youyoumu",
			Tags::IaMP	=> "07.5 - Suimusou",
			Tags::IN	=> "08 - Eiyashou",
			Tags::PoFV	=> "09 - Kaeidzuka",
			Tags::StB	=> "09.5 - Bunkachou",
			Tags::MoF	=> "10 - Fuujinroku",
			Tags::SWR	=> "10.5 - Hisouten",
			Tags::SA	=> "11 - Chireiden",
			Tags::UFO	=> "12 - Seirensen",
			Tags::soku	=> "12.3 - Hisoutensoku",
			Tags::DS	=> "12.5 - Bunkachou",
			Tags::GFW	=> "12.8 - Daisensou",
			Tags::TD	=> "13 - Shinreibyou",
			Tags::HM	=> "13.5 - Shinkirou",
			Tags::DDC	=> "14 - Kishinjou",
			Tags::ISC	=> "14.3 - Amanojaku",
			Tags::ULiL	=> "14.5 - Shinpiroku",
			Tags::LoLK	=> "15 - Kanjuden",
			Tags::AoCF	=> "15.5 - Hyouibana",
			Tags::HSiFS	=> "16 - Tenkuushou",
			Tags::VD	=> "16.5 - Hifuu Nightmare Diary",
			Tags::WBaWC	=> "17 - Kikeijuu",
			Tags::UM	=> "18 - Kouryuudou",
			// Stages
			Tags::st1	=> "",
			Tags::st2	=> "",
			Tags::st3	=> "",
			Tags::st4	=> "",
			Tags::st5	=> "",
			Tags::st6	=> "",
			Tags::ex	=> "",
		}
	}
}

// String to name utility
impl FromStr for Tags {
	type Err = ();
	fn from_str(input: &str) -> Result<Tags, Self::Err> {
		match input {
			"book"	=> Ok(Tags::book),
			"HRtP"	=> Ok(Tags::HRtP),
			"SoEW"	=> Ok(Tags::SoEW),
			"PoDD"	=> Ok(Tags::PoDD),
			"LLS"	=> Ok(Tags::LLS),
			"MS"	=> Ok(Tags::MS),
			"EoSD"	=> Ok(Tags::EoSD),
			"PCB"	=> Ok(Tags::PCB),
			"IaMP"	=> Ok(Tags::IaMP),
			"IN"	=> Ok(Tags::IN),
			"PoFV"	=> Ok(Tags::PoFV),
			"StB"	=> Ok(Tags::StB),
			"MoF"	=> Ok(Tags::MoF),
			"SWR"	=> Ok(Tags::SWR),
			"SA"	=> Ok(Tags::SA),
			"UFO"	=> Ok(Tags::UFO),
			"soku"	=> Ok(Tags::soku),
			"DS"	=> Ok(Tags::DS),
			"GFW"	=> Ok(Tags::GFW),
			"TD"	=> Ok(Tags::TD),
			"HM"	=> Ok(Tags::HM),
			"DDC"	=> Ok(Tags::DDC),
			"ISC"	=> Ok(Tags::ISC),
			"ULiL"	=> Ok(Tags::ULiL),
			"LoLK"	=> Ok(Tags::LoLK),
			"AoCF"	=> Ok(Tags::AoCF),
			"HSiFS"	=> Ok(Tags::HSiFS),
			"VD"	=> Ok(Tags::VD),
			"WBaWC"	=> Ok(Tags::WBaWC),
			"UM"	=> Ok(Tags::UM),
			// Stages
			"st1"	=> Ok(Tags::st1),
			"st2"	=> Ok(Tags::st2),
			"st3"	=> Ok(Tags::st3),
			"st4"	=> Ok(Tags::st4),
			"st5"	=> Ok(Tags::st5),
			"st6"	=> Ok(Tags::st6),
			"ex"	=> Ok(Tags::ex),
			_	=> Err(()),
		}
	}
}