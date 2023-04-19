use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Trackers {
    GoogleAnalytics,
    GoogleTagManager,
    FacebookPixel,
    Mixpanel,
}

pub struct PageTrackers {
    pub trackers: HashMap<Trackers, Vec<String>>,
}

impl PageTrackers {
    pub fn from_html(html: &str) -> Self {
        let mut trackers = HashMap::new();

        let ga_pattern = r"UA-\d+-\d+|G-[A-Z0-9]+";
        let gtm_pattern = r"(?i)GTM-[A-Z0-9]+";
        let fb_pixel_pattern = r"(?i)fbq\('init', '(\d+)'\)";
        let mixpanel_pattern = r"(?i)mixpanel\.init\('([a-z0-9]+)'\)";

        let ga_regex = Regex::new(ga_pattern).unwrap();
        let gtm_regex = Regex::new(gtm_pattern).unwrap();
        let fb_pixel_regex = Regex::new(fb_pixel_pattern).unwrap();
        let mixpanel_regex = Regex::new(mixpanel_pattern).unwrap();

        trackers.insert(
            Trackers::GoogleAnalytics,
            ga_regex
                .captures_iter(html)
                .map(|cap| cap[0].to_owned())
                .collect(),
        );
        trackers.insert(
            Trackers::GoogleTagManager,
            gtm_regex
                .captures_iter(html)
                .map(|cap| cap[0].to_owned())
                .collect(),
        );
        trackers.insert(
            Trackers::FacebookPixel,
            fb_pixel_regex
                .captures_iter(html)
                .map(|cap| cap[1].to_owned())
                .collect(),
        );
        trackers.insert(
            Trackers::Mixpanel,
            mixpanel_regex
                .captures_iter(html)
                .map(|cap| cap[1].to_owned())
                .collect(),
        );

        Self { trackers }
    }
}
