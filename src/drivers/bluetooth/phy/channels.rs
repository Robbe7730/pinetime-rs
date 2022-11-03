pub struct Channel {
    pub frequency: usize,
    pub number: u8,
}

impl Channel {
    pub fn new(number: u8) -> Self {
        let frequency = match number {
            37 => 2402,
             0 => 2404,
             1 => 2406,
             2 => 2408,
             3 => 2410,
             4 => 2412,
             5 => 2414,
             6 => 2416,
             7 => 2418,
             8 => 2420,
             9 => 2422,
            10 => 2424,
            38 => 2426,
            11 => 2428,
            12 => 2430,
            13 => 2432,
            14 => 2434,
            15 => 2436,
            16 => 2438,
            17 => 2440,
            18 => 2442,
            19 => 2444,
            20 => 2446,
            21 => 2448,
            22 => 2450,
            23 => 2452,
            24 => 2454,
            25 => 2456,
            26 => 2458,
            27 => 2460,
            28 => 2462,
            29 => 2464,
            30 => 2466,
            31 => 2468,
            32 => 2470,
            33 => 2472,
            34 => 2474,
            35 => 2476,
            36 => 2478,
            39 => 2480,
            x => panic!("Invalid channel {}", x)
        };

        return Self {
            frequency,
            number,
        }
    }

    pub fn whitening_iv(&self) -> u8 {
        return self.number | (1 << 6); // 1 << 6 is hard-wired but it is better to explicitly state it
    }

    pub fn is_advertising(&self) -> bool {
        return self.number >= 37;
    }
}
