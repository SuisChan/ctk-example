use protobuf::{Enum, Message, MessageField};
use protocol::{
    ChallengeAnswer, ChallengeType, ClientTokenRequest, ClientTokenRequestType,
    ClientTokenResponse, ClientTokenResponseType, GrantedTokenResponse, NativeAndroidData,
    PlatformSpecificData, Screen,
};
use reqwest::Method;

use crate::{hex, request};

const CLIENT_ID: &str = "9a8d2f0ce77a4e248bb71fefcb557637";
const DEVICE_ID: &str = "74d8baefc2e81eb2";

const CLIENT_VERSION: &str = "8.6.26.901";

const ANDROID_VERSION: &str = "10";
const API_VERSION: i32 = 29;
const DEVICE_NAME: &str = "Pixel";
const MODEL_STR: &str = "Pixel 4a";
const VENDOR: &str = "Google";
const UNKNOWN_VALUE: i32 = 32;

const URL: &str = "https://clienttoken.spotify.com/v1/clienttoken";

pub fn acquire_token() -> GrantedTokenResponse {
    let body = client_data_request();
    let response_bytes = request(Method::POST, URL, Some(body));

    let response =
        ClientTokenResponse::parse_from_bytes(&response_bytes).expect("failed to parse response");
    let typ = ClientTokenResponseType::from_i32(response.response_type.value())
        .expect("failed to parse response type");

    // ensure the response is valid
    if !typ.eq(&ClientTokenResponseType::RESPONSE_CHALLENGES_RESPONSE) {
        panic!("invalid response type");
    }

    let challenges_response = response.challenges().clone();

    let state = challenges_response.state.clone();
    let mut challenge = challenges_response
        .challenges
        .get(0)
        .expect("no challenges")
        .to_owned();

    let hash_cash_challenge = challenge.take_evaluate_hashcash_parameters();

    let ctx = vec![];
    let prefix = hex::decode(&hash_cash_challenge.prefix);
    let length = hash_cash_challenge.length;

    let mut suffix = vec![0; 0x10];
    let elapsed = crate::solve_hash_cash(&ctx, &prefix, length, &mut suffix).expect("must be ok");
    println!("elapsed: {elapsed:?}");

    // NOTE: that the suffix must be in uppercase (otherwise the server will reject it)
    let suffix = hex::encode(suffix).to_uppercase();

    let body = challenge_answer_request(&suffix, &state);
    let response_bytes = request(Method::POST, URL, Some(body));
    let response =
        ClientTokenResponse::parse_from_bytes(&response_bytes).expect("failed to parse response");
    let typ = ClientTokenResponseType::from_i32(response.response_type.value())
        .expect("failed to parse response type");

    // ensure the response is valid
    if !typ.eq(&ClientTokenResponseType::RESPONSE_GRANTED_TOKEN_RESPONSE) {
        panic!("invalid response type");
    }

    let granted_token_response = response.granted_token().clone();

    granted_token_response
}

fn client_data_request() -> Vec<u8> {
    let mut request = ClientTokenRequest {
        request_type: ClientTokenRequestType::REQUEST_CLIENT_DATA_REQUEST.into(),
        ..Default::default()
    };

    let client_data = request.mut_client_data();

    client_data.client_version = CLIENT_VERSION.into();
    client_data.client_id = CLIENT_ID.to_string();

    let sdk_data = client_data.mut_connectivity_sdk_data();
    sdk_data.device_id = DEVICE_ID.to_string();

    let mut platform_specific_data = PlatformSpecificData::new();

    let mut android = NativeAndroidData {
        android_version: ANDROID_VERSION.into(),
        api_version: API_VERSION,
        device_name: DEVICE_NAME.into(),
        model_str: MODEL_STR.into(),
        vendor: VENDOR.into(),
        vendor_2: VENDOR.into(),
        unknown_value_8: UNKNOWN_VALUE,
        ..Default::default()
    };

    android.screen_dimensions = MessageField::some(Screen {
        width: 1440,
        height: 2392,
        density: 411,
        unknown_value_4: 560,
        unknown_value_5: 560,
        ..Default::default()
    });

    platform_specific_data.set_android(android);
    sdk_data.platform_specific_data = MessageField::some(platform_specific_data);

    request.write_to_bytes().expect("failed to write request")
}

fn challenge_answer_request(suffix: &str, state: &str) -> Vec<u8> {
    let mut request = ClientTokenRequest {
        request_type: ClientTokenRequestType::REQUEST_CHALLENGE_ANSWERS_REQUEST.into(),
        ..Default::default()
    };

    let challenge_answers = request.mut_challenge_answers();

    let mut challenge_answer = ChallengeAnswer::new();
    challenge_answer.mut_hash_cash().suffix = suffix.to_string();
    challenge_answer.ChallengeType = ChallengeType::CHALLENGE_HASH_CASH.into();

    challenge_answers.state = state.to_string();
    challenge_answers.answers.push(challenge_answer);

    request.write_to_bytes().expect("failed to write request")
}
