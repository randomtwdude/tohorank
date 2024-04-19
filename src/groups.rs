// Character groups for detailed stats
// Generated from tohosort's dataset: 2019-11-26

use serde::{Serialize, Deserialize};
use std::str::FromStr;
use strum_macros::EnumIter;

// Tell the compiler to stop complaining
#[allow(non_camel_case_types)]

// Group by the work they appeared in
// taken from tohosort

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Clone, EnumIter)]
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
	ibun,
	UM,
	BM100,
	UDoALG,
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
	pub fn name(&self) -> &'static str {
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
			Tags::ibun	=> "Touhou Gouyoku Ibun",
			Tags::UM	=> "Unconnected Marketeers",
			Tags::BM100	=> "100th Black Market",
			Tags::UDoALG	=> "Unfinished Dream of All Living Ghost",
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
	pub fn exname(&self) -> &'static str {
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
			Tags::ibun	=> "17.5 - Gouyoku Ibun",
			Tags::UM	=> "18 - Kouryuudou",
			Tags::BM100	=> "18.5 - Bulletphilia-tachi no Yami-Ichiba",
			Tags::UDoALG	=> "19 - Unfinished Dream of All Living Ghost",
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
	// Returns true if it's a series tag
	pub fn is_series_tag(&self) -> bool {
		match self {
			Tags::st1	=> false,
			Tags::st2	=> false,
			Tags::st3	=> false,
			Tags::st4	=> false,
			Tags::st5	=> false,
			Tags::st6	=> false,
			Tags::ex	=> false,
			_	=> true,
		}
	}
}

// String to name utility
impl FromStr for Tags {
	type Err = ();
	fn from_str(input: &str) -> Result<Tags, Self::Err> {
		match input.to_lowercase().as_str() {
			"book"	=> Ok(Tags::book),
			"hrtp"	| "th01"	=> Ok(Tags::HRtP),
			"soew"	| "th02"	=> Ok(Tags::SoEW),
			"podd"	| "th03"	=> Ok(Tags::PoDD),
			"lls"	| "th04"	=> Ok(Tags::LLS),
			"ms"	| "th05"	=> Ok(Tags::MS),
			"eosd"	| "th06"	=> Ok(Tags::EoSD),
			"pcb"	| "th07"	=> Ok(Tags::PCB),
			"iamp"	| "th07.5"	=> Ok(Tags::IaMP),
			"in"	| "th08"	=> Ok(Tags::IN),
			"pofv"	| "th09"	=> Ok(Tags::PoFV),
			"stb"	| "th09.5"	=> Ok(Tags::StB),
			"mof"	| "th10"	=> Ok(Tags::MoF),
			"swr"	| "th10.5"	=> Ok(Tags::SWR),
			"sa"	| "th11"	=> Ok(Tags::SA),
			"ufo"	| "th12"	=> Ok(Tags::UFO),
			"soku"	| "th12.3"	=> Ok(Tags::soku),
			"ds"	| "th12.5"	=> Ok(Tags::DS),
			"gfw"	| "th12.8"	=> Ok(Tags::GFW),
			"td"	| "th13"	=> Ok(Tags::TD),
			"hm"	| "th13.5"	=> Ok(Tags::HM),
			"ddc"	| "th14"	=> Ok(Tags::DDC),
			"isc"	| "th14.3"	=> Ok(Tags::ISC),
			"ulil"	| "th14.5"	=> Ok(Tags::ULiL),
			"lolk"	| "th15"	=> Ok(Tags::LoLK),
			"aocf"	| "th15.5"	=> Ok(Tags::AoCF),
			"hsifs"	| "th16"	=> Ok(Tags::HSiFS),
			"vd"	| "th16.5"	=> Ok(Tags::VD),
			"wbawc"	| "th17"	=> Ok(Tags::WBaWC),
			"ibun"	| "th17.5"	=> Ok(Tags::ibun),
			"um"	| "th18"	=> Ok(Tags::UM),
			"bm100"	| "th18.5"	=> Ok(Tags::BM100),
			"udoalg"	| "th19"	=> Ok(Tags::UDoALG),
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