pub fn event_type_name(event_type: u16) -> &'static str {
    match event_type {
        0 => "EV_SYN",
        1 => "EV_KEY",
        2 => "EV_REL",
        3 => "EV_ABS",
        4 => "EV_MSC",
        5 => "EV_SW",
        6 => "EV_LED",
        7 => "EV_SND",
        8 => "EV_REP",
        9 => "EV_FF",
        10 => "EV_PWR",
        11 => "EV_FF_STATUS",
        _ => "EV_UNKNOWN",
    }
}

pub fn event_code_name(event_type: u16, event_code: u16) -> &'static str {
    match event_type {
        // EV_SYN
        0 => match event_code {
            0 => "SYN_REPORT",
            1 => "SYN_CONFIG",
            2 => "SYN_MT_REPORT",
            3 => "SYN_DROPPED",
            _ => "SYN_UNKNOWN",
        },

        // EV_KEY
        1 => match event_code {
            272 => "BTN_LEFT",
            273 => "BTN_RIGHT",
            274 => "BTN_MIDDLE",
            _ => "KEY_UNKNOWN",
        },

        // EV_REL
        2 => match event_code {
            0 => "REL_X",
            1 => "REL_Y",
            6 => "REL_HWHEEL",
            8 => "REL_WHEEL",
            11 => "REL_WHEEL_HI_RES",
            12 => "REL_HWHEEL_HI_RES",
            _ => "REL_UNKNOWN",
        },

        // EV_ABS
        3 => match event_code {
            0 => "ABS_X",
            1 => "ABS_Y",
            47 => "ABS_MT_SLOT",
            48 => "ABS_MT_TOUCH_MAJOR",
            49 => "ABS_MT_TOUCH_MINOR",
            52 => "ABS_MT_ORIENTATION",
            53 => "ABS_MT_POSITION_X",
            54 => "ABS_MT_POSITION_Y",
            55 => "ABS_MT_TOOL_TYPE",
            56 => "ABS_MT_BLOB_ID",
            57 => "ABS_MT_TRACKING_ID",
            _ => "ABS_UNKNOWN",
        },

        _ => "CODE_UNKNOWN",
    }
}
