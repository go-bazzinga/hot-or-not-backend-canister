use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{memory_manager::VirtualMemory, DefaultMemoryImpl};
use serde::Serialize;
use std::{
    collections::HashSet,
    time::{Duration, SystemTime},
};

use crate::{
    canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontend,
    common::types::{app_primitive_type::PostId, top_posts::post_score_index_item::PostStatus, utility_token::token_event::NewSlotType},
};

use super::hot_or_not::{
    BetDetails, BettingStatus, BettingStatusV1, GlobalBetId, GlobalRoomId, GlobalRoomIdV1, HotOrNotDetails, RoomDetailsV1, SlotDetailsV1, SlotId, StablePrincipal
};

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct Post {
    pub id: u64,
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub status: PostStatus,
    pub created_at: SystemTime,
    pub likes: HashSet<Principal>,
    pub share_count: u64,
    pub view_stats: PostViewStatistics,
    pub home_feed_score: FeedScore,
    pub creator_consent_for_inclusion_in_hot_or_not: bool,
    pub hot_or_not_details: Option<HotOrNotDetails>,
    #[serde(default)]
    pub is_nsfw: bool,
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct FeedScore {
    pub current_score: u64,
    pub last_synchronized_score: u64,
    pub last_synchronized_at: SystemTime,
}

impl Default for FeedScore {
    fn default() -> Self {
        FeedScore {
            current_score: 0,
            last_synchronized_score: 0,
            last_synchronized_at: SystemTime::UNIX_EPOCH,
        }
    }
}

#[derive(Deserialize, CandidType)]
pub enum PostViewDetailsFromFrontend {
    WatchedPartially {
        percentage_watched: u8,
    },
    WatchedMultipleTimes {
        // * only send complete watches as part of this count
        watch_count: u8,
        percentage_watched: u8,
    },
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize, Default)]
pub struct PostViewStatistics {
    pub total_view_count: u64,
    pub threshold_view_count: u64,
    pub average_watch_percentage: u8,
}

#[deprecated(note = "use PostDetailsForFrontendV1 instead")]
#[derive(Serialize, CandidType, Deserialize, Debug, PartialEq, Eq)]
pub struct PostDetailsForFrontend {
    pub id: u64,
    pub created_by_display_name: Option<String>,
    pub created_by_unique_user_name: Option<String>,
    pub created_by_user_principal_id: Principal,
    pub created_by_profile_photo_url: Option<String>,
    pub created_at: SystemTime,
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub status: PostStatus,
    pub total_view_count: u64,
    pub like_count: u64,
    pub liked_by_me: bool,
    pub home_feed_ranking_score: u64,
    pub hot_or_not_feed_ranking_score: Option<u64>,
    pub hot_or_not_betting_status: Option<BettingStatus>,
    pub is_nsfw: bool,
}

#[derive(Serialize, CandidType, Deserialize, Debug, PartialEq, Eq)]
pub struct PostDetailsForFrontendV1 {
    pub id: u64,
    pub created_by_display_name: Option<String>,
    pub created_by_unique_user_name: Option<String>,
    pub created_by_user_principal_id: Principal,
    pub created_by_profile_photo_url: Option<String>,
    pub created_at: SystemTime,
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub status: PostStatus,
    pub total_view_count: u64,
    pub like_count: u64,
    pub liked_by_me: bool,
    pub home_feed_ranking_score: u64,
    pub hot_or_not_feed_ranking_score: Option<u64>,
    pub hot_or_not_betting_status_v1: Option<BettingStatusV1>,
    pub is_nsfw: bool,
}

#[derive(Serialize, CandidType, Deserialize)]
pub struct PostDetailsFromFrontend {
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub creator_consent_for_inclusion_in_hot_or_not: bool,
    pub is_nsfw: bool,
}

impl From<Post> for PostDetailsFromFrontend {
    fn from(value: Post) -> Self {
        PostDetailsFromFrontend {
            description: value.description,
            hashtags: value.hashtags,
            video_uid: value.video_uid,
            creator_consent_for_inclusion_in_hot_or_not: value
                .creator_consent_for_inclusion_in_hot_or_not,
            is_nsfw: value.is_nsfw,
        }
    }
}

impl Post {
    pub fn add_view_details(&mut self, details: &PostViewDetailsFromFrontend) {
        match details {
            PostViewDetailsFromFrontend::WatchedPartially { percentage_watched } => {
                assert!(*percentage_watched <= 100 && *percentage_watched > 0);
                self.view_stats.average_watch_percentage =
                    self.recalculate_average_watched(*percentage_watched, 0);
                self.view_stats.total_view_count += 1;
                if *percentage_watched > 20 {
                    self.view_stats.threshold_view_count += 1;
                }
            }
            PostViewDetailsFromFrontend::WatchedMultipleTimes {
                watch_count,
                percentage_watched,
            } => {
                assert!(*percentage_watched <= 100 && *percentage_watched > 0);
                self.view_stats.average_watch_percentage =
                    self.recalculate_average_watched(*percentage_watched, *watch_count);
                self.view_stats.total_view_count += (*watch_count + 1) as u64;
                self.view_stats.threshold_view_count += *watch_count as u64;
                if *percentage_watched > 20 {
                    self.view_stats.threshold_view_count += 1;
                }
            }
        }
    }

    #[deprecated(note = "use get_post_details_for_frontend_for_this_post_v1 instead")]
    pub fn get_post_details_for_frontend_for_this_post(
        &self,
        user_profile: UserProfileDetailsForFrontend,
        caller: Principal,
        current_time: &SystemTime,
        room_details_map: &ic_stable_structures::btreemap::BTreeMap<
            GlobalRoomId,
            RoomDetailsV1,
            VirtualMemory<DefaultMemoryImpl>,
        >,
        post_principal_map: &ic_stable_structures::btreemap::BTreeMap<
            (PostId, StablePrincipal),
            (),
            VirtualMemory<DefaultMemoryImpl>,
        >,
        slot_details_map: &ic_stable_structures::btreemap::BTreeMap<
            (PostId, SlotId),
            SlotDetailsV1,
            VirtualMemory<DefaultMemoryImpl>,
        >,
    ) -> PostDetailsForFrontend {
        PostDetailsForFrontend {
            id: self.id,
            created_by_display_name: user_profile.display_name,
            created_by_unique_user_name: user_profile.unique_user_name,
            created_by_user_principal_id: user_profile.principal_id,
            created_by_profile_photo_url: user_profile.profile_picture_url,
            created_at: self.created_at,
            description: self.description.clone(),
            hashtags: self.hashtags.clone(),
            video_uid: self.video_uid.clone(),
            status: self.status.clone(),
            total_view_count: self.view_stats.total_view_count,
            like_count: self.likes.len() as u64,
            is_nsfw: self.is_nsfw,
            liked_by_me: self.likes.contains(&caller),
            home_feed_ranking_score: self.home_feed_score.current_score,
            hot_or_not_feed_ranking_score: if self.hot_or_not_details.is_some() {
                Some(
                    self.hot_or_not_details
                        .as_ref()
                        .unwrap()
                        .hot_or_not_feed_score
                        .current_score,
                )
            } else {
                None
            },
            hot_or_not_betting_status: if self.creator_consent_for_inclusion_in_hot_or_not {
                Some(self.get_hot_or_not_betting_status_for_this_post_v1(
                    current_time,
                    &caller,
                    room_details_map,
                    post_principal_map,
                    slot_details_map,
                ))
            } else {
                None
            },
        }
    }


    pub fn get_post_details_for_frontend_for_this_post_v1(
        &self,
        user_profile: UserProfileDetailsForFrontend,
        caller: Principal,
        current_time: &SystemTime,
        room_details_map: &ic_stable_structures::btreemap::BTreeMap<
            GlobalRoomIdV1,
            RoomDetailsV1,
            VirtualMemory<DefaultMemoryImpl>,
        >,
        post_principal_map: &ic_stable_structures::btreemap::BTreeMap<
            (PostId, StablePrincipal),
            (),
            VirtualMemory<DefaultMemoryImpl>,
        >,
        slot_details_map: &ic_stable_structures::btreemap::BTreeMap<
            (PostId, NewSlotType),
            SlotDetailsV1,
            VirtualMemory<DefaultMemoryImpl>,
        >,
    ) -> PostDetailsForFrontendV1 {
        PostDetailsForFrontendV1 {
            id: self.id,
            created_by_display_name: user_profile.display_name,
            created_by_unique_user_name: user_profile.unique_user_name,
            created_by_user_principal_id: user_profile.principal_id,
            created_by_profile_photo_url: user_profile.profile_picture_url,
            created_at: self.created_at,
            description: self.description.clone(),
            hashtags: self.hashtags.clone(),
            video_uid: self.video_uid.clone(),
            status: self.status.clone(),
            total_view_count: self.view_stats.total_view_count,
            like_count: self.likes.len() as u64,
            is_nsfw: self.is_nsfw,
            liked_by_me: self.likes.contains(&caller),
            home_feed_ranking_score: self.home_feed_score.current_score,
            hot_or_not_feed_ranking_score: if self.hot_or_not_details.is_some() {
                Some(
                    self.hot_or_not_details
                        .as_ref()
                        .unwrap()
                        .hot_or_not_feed_score
                        .current_score,
                )
            } else {
                None
            },
            hot_or_not_betting_status_v1: if self.creator_consent_for_inclusion_in_hot_or_not {
                Some(self.get_hot_or_not_betting_status_for_this_post_v2(
                    current_time,
                    &caller,
                    room_details_map,
                    post_principal_map,
                    slot_details_map,
                ))
            } else {
                None
            },
        }
    }


    pub fn increment_share_count(&mut self) -> u64 {
        self.share_count += 1;
        self.share_count
    }

    pub fn new(
        id: u64,
        post_details_from_frontend: &PostDetailsFromFrontend,
        current_time: &SystemTime,
    ) -> Self {
        Post {
            id,
            description: (*post_details_from_frontend.description).to_string(),
            hashtags: (*post_details_from_frontend.hashtags).to_vec(),
            video_uid: (*post_details_from_frontend.video_uid).to_string(),
            status: PostStatus::Uploaded,
            created_at: *current_time,
            likes: HashSet::new(),
            share_count: 0,
            is_nsfw: post_details_from_frontend.is_nsfw,
            view_stats: PostViewStatistics {
                total_view_count: 0,
                threshold_view_count: 0,
                average_watch_percentage: 0,
            },
            home_feed_score: FeedScore::default(),
            creator_consent_for_inclusion_in_hot_or_not: post_details_from_frontend
                .creator_consent_for_inclusion_in_hot_or_not,
            hot_or_not_details: if post_details_from_frontend
                .creator_consent_for_inclusion_in_hot_or_not
            {
                Some(HotOrNotDetails::default())
            } else {
                None
            },
        }
    }

    fn recalculate_average_watched(&self, percentage_watched: u8, full_view_count: u8) -> u8 {
        let earlier_sum_component =
            self.view_stats.average_watch_percentage as u64 * self.view_stats.total_view_count;
        let current_full_view_component = 100 * full_view_count as u64;
        let current_total_dividend =
            earlier_sum_component + current_full_view_component + percentage_watched as u64;
        let current_total_divisor = self.view_stats.total_view_count + full_view_count as u64 + 1;

        (current_total_dividend / current_total_divisor) as u8
    }

    pub fn recalculate_home_feed_score(&mut self, current_time: &SystemTime) {
        // ic_cdk::print(std::format!(
        //     "🥫 post from home feed score recalculation: {:?}",
        //     self
        // ));
        let likes_component = match self.view_stats.total_view_count {
            0 => 0,
            _ => (1000 * 10 * self.likes.len() as u64) / self.view_stats.total_view_count,
        };
        // println!("🥫 likes_component: {}", likes_component);
        let threshold_views_component = match self.view_stats.total_view_count {
            0 => 0,
            _ => (1000 * self.view_stats.threshold_view_count) / self.view_stats.total_view_count,
        };
        // println!(
        //     "🥫 threshold_views_component: {}",
        //     threshold_views_component
        // );
        let average_percent_viewed_component = 10 * self.view_stats.average_watch_percentage as u64;
        // println!(
        //     "🥫 average_percent_viewed_component: {}",
        //     average_percent_viewed_component
        // );

        // println!("🥫 share_count: {}", self.share_count);
        // println!("🥫 total_view_count: {}", self.view_stats.total_view_count);
        let post_share_component = match self.view_stats.total_view_count {
            0 => 0,
            _ => (1000 * 100 * self.share_count) / self.view_stats.total_view_count,
        };
        // println!("🥫 post_share_component: {}", post_share_component);

        let age_of_video_in_hours = (current_time
            .duration_since(self.created_at)
            .unwrap_or(Duration::ZERO)
            .as_secs())
            / (60 * 60);
        let subtracting_factor = age_of_video_in_hours / 4;
        // println!("🥫 subtracting_factor: {}", subtracting_factor);
        let subtraction_outcome = 1000 - (50 * subtracting_factor as i64);
        // println!("🥫 subtraction_outcome: {}", subtraction_outcome);
        let mut age_of_video_component = subtraction_outcome.max(0) as u64;
        // println!("🥫 age_of_video_component: {}", age_of_video_component);
        if age_of_video_in_hours <= 16 {
            age_of_video_component *= 3;
        }
        // println!("🥫 age_of_video_component: {}", age_of_video_component);

        let hot_or_not_participation_component = match self.hot_or_not_details {
            Some(ref details) => {
                let total_hot_or_not_participations =
                    details.aggregate_stats.total_number_of_hot_bets
                        + details.aggregate_stats.total_number_of_not_bets;

                match total_hot_or_not_participations {
                    0 => 0,
                    _ => {
                        (1000 * details.aggregate_stats.total_number_of_hot_bets)
                            / total_hot_or_not_participations
                    }
                }
            }
            None => 0,
        };
        // println!(
        //     "🥫 hot_or_not_participation_component: {}",
        //     hot_or_not_participation_component
        // );

        self.home_feed_score.current_score = likes_component
            + threshold_views_component
            + average_percent_viewed_component
            + post_share_component
            + age_of_video_component
            + hot_or_not_participation_component;
    }

    pub fn recalculate_hot_or_not_feed_score(&mut self, current_time: &SystemTime) {
        if self.hot_or_not_details.is_some() {
            let likes_component = match self.view_stats.total_view_count {
                0 => 0,
                _ => (1000 * 10 * self.likes.len() as u64) / self.view_stats.total_view_count,
            };
            // println!("🥫 likes_component: {}", likes_component);
            let threshold_views_component = match self.view_stats.total_view_count {
                0 => 0,
                _ => {
                    (1000 * self.view_stats.threshold_view_count) / self.view_stats.total_view_count
                }
            };
            // println!(
            //     "🥫 threshold_views_component: {}",
            //     threshold_views_component
            // );
            let average_percent_viewed_component =
                10 * self.view_stats.average_watch_percentage as u64;
            // println!(
            //     "🥫 average_percent_viewed_component: {}",
            //     average_percent_viewed_component
            // );
            let post_share_component = match self.view_stats.total_view_count {
                0 => 0,
                _ => (1000 * 100 * self.share_count) / self.view_stats.total_view_count,
            };
            // println!("🥫 post_share_component: {}", post_share_component);

            let age_of_video_in_hours = (current_time
                .duration_since(self.created_at)
                .unwrap_or(Duration::ZERO)
                .as_secs())
                / (60 * 60);
            let subtracting_factor = age_of_video_in_hours / 4;
            // println!("🥫 subtracting_factor: {}", subtracting_factor);
            let mut age_of_video_component =
                (1000 - 50 * subtracting_factor as isize).max(0) as u64;
            // println!("🥫 age_of_video_component: {}", age_of_video_component);
            if age_of_video_in_hours <= 16 {
                age_of_video_component *= 3;
            }
            // println!("🥫 age_of_video_component: {}", age_of_video_component);

            let hot_or_not_score_component = match self.hot_or_not_details {
                Some(ref details) => {
                    let total_hot_or_not_participations =
                        details.aggregate_stats.total_number_of_hot_bets
                            + details.aggregate_stats.total_number_of_not_bets;

                    match total_hot_or_not_participations {
                        0 => 0,
                        _ => {
                            2 * (1000
                                - 2 * ((1000 * details.aggregate_stats.total_number_of_hot_bets)
                                    / total_hot_or_not_participations)
                                    .abs_diff(500))
                        }
                    }
                }
                None => 0,
            };
            // println!(
            //     "🥫 hot_or_not_score_component: {}",
            //     hot_or_not_score_component
            // );

            self.hot_or_not_details
                .as_mut()
                .unwrap()
                .hot_or_not_feed_score
                .current_score = likes_component
                + threshold_views_component
                + average_percent_viewed_component
                + post_share_component
                + age_of_video_component
                + hot_or_not_score_component;
        }
    }

    pub fn toggle_like_status(&mut self, user_principal_id: &Principal) -> bool {
        // if liked, return true & if unliked, return false
        if self.likes.contains(user_principal_id) {
            self.likes.remove(user_principal_id);
            false
        } else {
            self.likes.insert(*user_principal_id);
            true
        }
    }

    pub fn update_status(&mut self, status: PostStatus) {
        self.status = status;
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, time::Instant};

    use crate::canister_specific::individual_user_template::types::hot_or_not::{
        test_hot_or_not::setup_room_and_bet_details_map, BetDirection,
    };

    use super::*;

    #[test]
    fn test_new() {
        let post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "This is a new post".to_string(),
                hashtags: vec!["#fun".to_string(), "#post".to_string()],
                video_uid: "abcd1234".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: false,
                is_nsfw: false,
            },
            &SystemTime::now(),
        );

        assert!(post.hot_or_not_details.is_none());

        let post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "This is a new post".to_string(),
                hashtags: vec!["#fun".to_string(), "#post".to_string()],
                video_uid: "abcd1234".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &SystemTime::now(),
        );

        assert!(post.hot_or_not_details.is_some());
    }

    #[test]
    fn test_recalculate_home_feed_score_case_1() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_423_915))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 150;
        (0..29).for_each(|number| {
            post.likes.insert(Principal::from_slice(&[number]));
        });
        assert_eq!(post.likes.len(), 29);
        post.share_count = 3;
        post.view_stats.threshold_view_count = 130;
        post.view_stats.average_watch_percentage = 59;

        (0..80).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            80
        );

        (80..145).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            65
        );

        post.recalculate_home_feed_score(&recalculation_time);

        println!("🧪 Homefeed score: {}", post.home_feed_score.current_score);
        assert_eq!(post.home_feed_score.current_score, 8_790);
    }

    #[test]
    fn test_recalculate_home_feed_score_case_2() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_293_841))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 400;
        (0..28).for_each(|number| {
            post.likes.insert(Principal::from_slice(&[number]));
        });
        assert_eq!(post.likes.len(), 28);
        post.share_count = 4;
        post.view_stats.threshold_view_count = 340;
        post.view_stats.average_watch_percentage = 47;

        (0..216).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            216
        );

        (216..360).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            144
        );

        post.recalculate_home_feed_score(&recalculation_time);

        println!("🧪 Homefeed score: {}", post.home_feed_score.current_score);
        assert_eq!(post.home_feed_score.current_score, 4_120);
    }

    #[test]
    fn test_recalculate_home_feed_score_case_3() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_105_696))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 3_500;
        (0..245).for_each(|number| {
            post.likes.insert(Principal::from_slice(&[number]));
        });
        assert_eq!(post.likes.len(), 245);
        post.share_count = 46;
        post.view_stats.threshold_view_count = 3_045;
        post.view_stats.average_watch_percentage = 54;

        (0..1_078).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            1_078
        );

        (1_078..2_695).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            1_617
        );

        post.recalculate_home_feed_score(&recalculation_time);

        println!("🧪 Homefeed score: {}", post.home_feed_score.current_score);
        assert_eq!(post.home_feed_score.current_score, 3_824);
    }

    #[test]
    fn test_recalculate_home_feed_score_case_4() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_677_615_537))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 15_000;
        (0..1_200).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 1_200);
        post.share_count = 180;
        post.view_stats.threshold_view_count = 10_200;
        post.view_stats.average_watch_percentage = 58;

        (0..4_725).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            4_725
        );

        (4_725..10_500).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            5_775
        );

        post.recalculate_home_feed_score(&recalculation_time);

        println!("🧪 Homefeed score: {}", post.home_feed_score.current_score);
        assert_eq!(post.home_feed_score.current_score, 3_710);
    }

    #[test]
    fn test_recalculate_home_feed_score_case_5() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_675_311_162))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 75_000;
        (0..3_000).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 3_000);
        post.share_count = 360;
        post.view_stats.threshold_view_count = 66_000;
        post.view_stats.average_watch_percentage = 59;

        (0..26_827).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            26_827
        );

        (26_827..54_750).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            27_923
        );

        post.recalculate_home_feed_score(&recalculation_time);

        println!("🧪 Homefeed score: {}", post.home_feed_score.current_score);
        assert_eq!(post.home_feed_score.current_score, 2_839);
    }

    #[test]
    fn test_recalculate_home_feed_score_case_6() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_436_004))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 150;
        (0..3).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 3);
        post.share_count = 0;
        post.view_stats.threshold_view_count = 49;
        post.view_stats.average_watch_percentage = 30;

        (0..18).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            18
        );

        (18..45).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            27
        );

        post.recalculate_home_feed_score(&recalculation_time);

        println!("🧪 Homefeed score: {}", post.home_feed_score.current_score);
        assert_eq!(post.home_feed_score.current_score, 4_226);
    }

    #[test]
    fn test_recalculate_home_feed_score_case_7() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_295_932))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 400;
        (0..4).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 4);
        post.share_count = 0;
        post.view_stats.threshold_view_count = 88;
        post.view_stats.average_watch_percentage = 24;

        (0..40).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            40
        );

        (40..68).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            28
        );

        post.recalculate_home_feed_score(&recalculation_time);

        println!("🧪 Homefeed score: {}", post.home_feed_score.current_score);
        assert_eq!(post.home_feed_score.current_score, 1_698);
    }

    #[test]
    fn test_recalculate_home_feed_score_case_8() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_005_696))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 3_500;
        (0..35).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 35);
        post.share_count = 0;
        post.view_stats.threshold_view_count = 1_190;
        post.view_stats.average_watch_percentage = 18;

        (0..466).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            466
        );

        (466..805).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            339
        );

        post.recalculate_home_feed_score(&recalculation_time);

        println!("🧪 Homefeed score: {}", post.home_feed_score.current_score);
        assert_eq!(post.home_feed_score.current_score, 1_198);
    }

    #[test]
    fn test_recalculate_home_feed_score_case_9() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_677_396_626))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 15_000;
        (0..600).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 600);
        post.share_count = 30;
        post.view_stats.threshold_view_count = 2_550;
        post.view_stats.average_watch_percentage = 50;

        (0..1_320).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            1_320
        );

        (1_320..2_400).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            1_080
        );

        post.recalculate_home_feed_score(&recalculation_time);

        println!("🧪 Homefeed score: {}", post.home_feed_score.current_score);
        assert_eq!(post.home_feed_score.current_score, 1_820);
    }

    #[test]
    fn test_recalculate_home_feed_score_case_10() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_673_117_006))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 75_000;
        (0..750).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 750);
        post.share_count = 15;
        post.view_stats.threshold_view_count = 21_750;
        post.view_stats.average_watch_percentage = 67;

        (0..6_270).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            6_270
        );

        (6_270..14_250).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            7_980
        );

        post.recalculate_home_feed_score(&recalculation_time);

        println!("🧪 Homefeed score: {}", post.home_feed_score.current_score);
        assert_eq!(post.home_feed_score.current_score, 1_520);
    }

    #[test]
    fn test_recalculate_hot_or_not_feed_score_case_1() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_423_915))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 150;
        (0..29).for_each(|number| {
            post.likes.insert(Principal::from_slice(&[number]));
        });
        assert_eq!(post.likes.len(), 29);
        post.share_count = 3;
        post.view_stats.threshold_view_count = 130;
        post.view_stats.average_watch_percentage = 59;

        (0..80).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            80
        );

        (80..145).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            65
        );

        post.recalculate_hot_or_not_feed_score(&recalculation_time);

        println!(
            "🧪 Hot or Not score: {}",
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score
        );
        assert_eq!(
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score,
            10_035
        );
    }

    #[test]
    fn test_recalculate_hot_or_not_feed_score_case_2() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_293_841))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 400;
        (0..28).for_each(|number| {
            post.likes.insert(Principal::from_slice(&[number]));
        });
        assert_eq!(post.likes.len(), 28);
        post.share_count = 4;
        post.view_stats.threshold_view_count = 340;
        post.view_stats.average_watch_percentage = 47;

        (0..216).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            216
        );

        (216..360).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            144
        );

        post.recalculate_hot_or_not_feed_score(&recalculation_time);

        println!(
            "🧪 Hot or Not feed score: {}",
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score
        );
        assert_eq!(
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score,
            5_120
        );
    }

    #[test]
    fn test_recalculate_hot_or_not_feed_score_case_3() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_105_696))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 3_500;
        (0..245).for_each(|number| {
            post.likes.insert(Principal::from_slice(&[number]));
        });
        assert_eq!(post.likes.len(), 245);
        post.share_count = 46;
        post.view_stats.threshold_view_count = 3_045;
        post.view_stats.average_watch_percentage = 54;

        (0..1_078).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            1_078
        );

        (1_078..2_695).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            1_617
        );

        post.recalculate_hot_or_not_feed_score(&recalculation_time);

        println!(
            "🧪 Hot or Not feed score: {}",
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score
        );
        assert_eq!(
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score,
            5_024
        );
    }

    #[test]
    fn test_recalculate_hot_or_not_feed_score_case_4() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_677_615_537))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 15_000;
        (0..1_200).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 1_200);
        post.share_count = 180;
        post.view_stats.threshold_view_count = 10_200;
        post.view_stats.average_watch_percentage = 58;

        (0..4_725).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            4_725
        );

        (4_725..10_500).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            5_775
        );

        post.recalculate_hot_or_not_feed_score(&recalculation_time);

        println!(
            "🧪 Hot or Not feed score: {}",
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score
        );
        assert_eq!(
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score,
            5_060
        );
    }

    #[test]
    fn test_recalculate_hot_or_not_feed_score_case_5() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_675_311_162))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 75_000;
        (0..3_000).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 3_000);
        post.share_count = 360;
        post.view_stats.threshold_view_count = 66_000;
        post.view_stats.average_watch_percentage = 59;

        (0..26_827).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            26_827
        );

        (26_827..54_750).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            27_923
        );

        post.recalculate_hot_or_not_feed_score(&recalculation_time);

        println!(
            "🧪 Hot or Not feed score: {}",
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score
        );
        assert_eq!(
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score,
            4_306
        );
    }

    #[test]
    fn test_recalculate_hot_or_not_feed_score_case_6() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_436_004))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 150;
        (0..3).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 3);
        post.share_count = 0;
        post.view_stats.threshold_view_count = 49;
        post.view_stats.average_watch_percentage = 30;

        (0..18).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            18
        );

        (18..45).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            27
        );

        post.recalculate_hot_or_not_feed_score(&recalculation_time);

        println!(
            "🧪 Hot or Not feed score: {}",
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score
        );
        assert_eq!(
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score,
            5_426
        );
    }

    #[test]
    fn test_recalculate_hot_or_not_feed_score_case_7() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_295_932))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 400;
        (0..4).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 4);
        post.share_count = 0;
        post.view_stats.threshold_view_count = 88;
        post.view_stats.average_watch_percentage = 24;

        (0..40).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            40
        );

        (40..68).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            28
        );

        post.recalculate_hot_or_not_feed_score(&recalculation_time);

        println!(
            "🧪 Hot or Not feed score: {}",
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score
        );
        assert_eq!(
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score,
            2_758
        );
    }

    #[test]
    fn test_recalculate_hot_or_not_feed_score_case_8() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_005_696))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 3_500;
        (0..35).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 35);
        post.share_count = 0;
        post.view_stats.threshold_view_count = 1_190;
        post.view_stats.average_watch_percentage = 18;

        (0..466).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            466
        );

        (466..805).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            339
        );

        post.recalculate_hot_or_not_feed_score(&recalculation_time);

        println!(
            "🧪 Hot or Not feed score: {}",
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score
        );
        assert_eq!(
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score,
            2_308
        );
    }

    #[test]
    fn test_recalculate_hot_or_not_feed_score_case_9() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_677_396_626))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 15_000;
        (0..600).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 600);
        post.share_count = 30;
        post.view_stats.threshold_view_count = 2_550;
        post.view_stats.average_watch_percentage = 50;

        (0..1_320).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            1_320
        );

        (1_320..2_400).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            1_080
        );

        post.recalculate_hot_or_not_feed_score(&recalculation_time);

        println!(
            "🧪 Hot or Not feed score: {}",
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score
        );
        assert_eq!(
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score,
            3_070
        );
    }

    #[test]
    fn test_recalculate_hot_or_not_feed_score_case_10() {
        // let guard = pprof::ProfilerGuardBuilder::default()
        //     .frequency(1000)
        //     .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        //     .build()
        //     .unwrap();
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_673_117_006))
            .unwrap();
        let betting_time = post_created_at
            .checked_add(Duration::from_secs(1_000))
            .unwrap();
        let recalculation_time = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_678_438_993))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );
        post.view_stats.total_view_count = 75_000;
        (0..750).for_each(|number| {
            post.likes.insert(Principal::self_authenticating(
                (number as usize).to_ne_bytes(),
            ));
        });
        assert_eq!(post.likes.len(), 750);
        post.share_count = 15;
        post.view_stats.threshold_view_count = 21_750;
        post.view_stats.average_watch_percentage = 67;

        (0..6_270).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Hot,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            if result.is_err() {
                println!("🧪 Error: {:?}", result);
            }
            assert!(result.is_ok());
        });
        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_hot_bets,
            6_270
        );

        (6_270..14_250).for_each(|number| {
            let result = post.place_hot_or_not_bet_v1(
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                &Principal::self_authenticating((number as usize).to_ne_bytes()),
                100,
                &BetDirection::Not,
                &betting_time,
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );
            assert!(result.is_ok());
        });

        assert_eq!(
            post.hot_or_not_details
                .clone()
                .unwrap()
                .aggregate_stats
                .total_number_of_not_bets,
            7_980
        );

        post.recalculate_hot_or_not_feed_score(&recalculation_time);

        println!(
            "🧪 Hot or Not feed score: {}",
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score
        );
        assert_eq!(
            post.hot_or_not_details
                .as_ref()
                .unwrap()
                .hot_or_not_feed_score
                .current_score,
            2_840
        );

        // if let Ok(report) = guard.report().build() {
        //     let file = File::create("flamegraph.svg").unwrap();
        //     report.flamegraph(file).unwrap();
        // };
    }

    #[test]
    fn test_recalculate_average_watched_case_1() {
        let post_created_at = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(1_673_117_006))
            .unwrap();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            },
            &post_created_at,
        );

        assert_eq!(post.view_stats.average_watch_percentage, 0);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedPartially {
            percentage_watched: 98,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 98);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedMultipleTimes {
            watch_count: 1,
            percentage_watched: 86,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 94);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedMultipleTimes {
            watch_count: 4,
            percentage_watched: 81,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 95);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedMultipleTimes {
            watch_count: 4,
            percentage_watched: 28,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 91);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedMultipleTimes {
            watch_count: 4,
            percentage_watched: 1,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 88);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedMultipleTimes {
            watch_count: 1,
            percentage_watched: 43,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 86);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedMultipleTimes {
            watch_count: 2,
            percentage_watched: 20,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 84);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedPartially {
            percentage_watched: 38,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 82);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedMultipleTimes {
            watch_count: 2,
            percentage_watched: 18,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 80);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedMultipleTimes {
            watch_count: 3,
            percentage_watched: 84,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 82);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedPartially {
            percentage_watched: 79,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 81);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedPartially {
            percentage_watched: 76,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 80);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedMultipleTimes {
            watch_count: 4,
            percentage_watched: 20,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 80);

        post.add_view_details(&PostViewDetailsFromFrontend::WatchedPartially {
            percentage_watched: 1,
        });

        assert_eq!(post.view_stats.average_watch_percentage, 77);
    }
}
