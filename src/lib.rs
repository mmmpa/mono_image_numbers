use bit_iterator::BitIterator;

pub struct MonoImageNumbers<P: SourceProvider, C: DataContainer> {
    height: u8,
    provider: P,
    container: C,
}

pub trait SourceProvider {
    fn pixels(&self, n: u8) -> &[u8];
    fn width(&self, n: u8) -> u8;
}

pub trait DataContainer {
    fn update(&mut self, index: usize, b: bool);
    fn data(&self) -> &[bool];
}

pub const MINUS: u8 = 254;
pub const PERIOD: u8 = 255;

impl<P: SourceProvider, C: DataContainer> MonoImageNumbers<P, C> {
    pub fn new(height: u8, provider: P, container: C) -> Self {
        Self {
            height,
            provider,
            container,
        }
    }

    fn each_digit(&self, n: isize) -> (usize, [(u8, u8); 16], usize) {
        let mut char_and_width = [(0, 0); 16];
        let mut length = 0;
        let mut now = n;
        let mut canvas_width = 0;
        let mut minus = false;

        if now < 0 {
            minus = true;
            now *= -1;
        }

        while now > 0 {
            let n = (now % 10) as u8;
            let w = self.provider.width(n);
            canvas_width += w;
            char_and_width[length] = (n, w);
            length += 1;
            now /= 10
        }

        if minus {
            let w = self.provider.width(MINUS);
            canvas_width += w;
            char_and_width[length] = (MINUS, w);
            length += 1;
        }

        (length, char_and_width, canvas_width as usize + (length - 1))
    }

    pub fn update_c(&mut self, each: (usize, [(u8, u8); 16], usize)) -> (usize, usize) {
        let (l, v, canvas_w) = each;
        let canvas_h = self.height as usize;
        let mut offset = 0;

        for (i, (char, char_width)) in v[0..l].iter().rev().enumerate() {
            let owned = self
                .provider
                .pixels(*char)
                .iter()
                .map(|n| *n)
                .flat_map(|n| BitIterator::from(n))
                .take(*char_width as usize * canvas_h)
                .collect::<Vec<_>>();

            owned
                .chunks(*char_width as usize)
                .into_iter()
                .enumerate()
                .for_each(|(y, row)| {
                    row.into_iter().enumerate().for_each(|(step_x, b)| {
                        self.container.update(y * canvas_w + offset + step_x, *b)
                    });
                });

            offset += *char_width as usize + 1;

            if i == l - 1 {
                break;
            }

            // 文字間のスペース分の古いピクセルを消す
            for y in 0..canvas_h {
                self.container.update(y * canvas_w + offset - 1, false)
            }
        }

        (canvas_w, canvas_h)
    }

    pub fn update(&mut self, n: isize) -> (usize, usize) {
        self.update_c(self.each_digit(n))
    }

    pub fn update_f(&mut self, f: f64, level: usize) -> (usize, usize) {
        let n = (f * (10_i32.pow(level as u32) as f64) as f64).floor() as isize;
        let (l, mut v, canvas_w) = self.each_digit(n);

        let w = self.provider.width(PERIOD);

        for i in (level..l + 1).into_iter().rev() {
            v[i + 1] = v[i];
        }

        v[level as usize] = (PERIOD, w);

        self.update_c((l + 1, v, canvas_w + w as usize + 1))
    }
}

#[cfg(test)]
mod tests {
    use crate::{DataContainer, MonoImageNumbers, SourceProvider, MINUS, PERIOD};
    use itertools::Itertools;

    const VEC_NUM_1: (u8, u8, [u8; 10]) = (3, 10, [0, 44, 151, 0, 0, 0, 0, 0, 0, 0]);
    const VEC_NUM_2: (u8, u8, [u8; 10]) = (5, 10, [0, 0, 232, 136, 159, 0, 0, 0, 0, 0]);
    const VEC_NUM_3: (u8, u8, [u8; 10]) = (5, 10, [0, 0, 232, 132, 193, 139, 128, 0, 0, 0]);
    const VEC_NUM_4: (u8, u8, [u8; 10]) = (5, 10, [0, 0, 35, 42, 95, 16, 128, 0, 0, 0]);
    const VEC_NUM_5: (u8, u8, [u8; 10]) = (5, 10, [0, 0, 244, 56, 33, 139, 128, 0, 0, 0]);
    const VEC_NUM_6: (u8, u8, [u8; 10]) = (5, 10, [34, 33, 232, 198, 46, 0, 0, 0, 0, 0]);
    const VEC_NUM_7: (u8, u8, [u8; 10]) = (5, 10, [0, 1, 248, 200, 68, 33, 0, 0, 0, 0]);
    const VEC_NUM_8: (u8, u8, [u8; 10]) = (5, 10, [116, 98, 232, 198, 46, 0, 0, 0, 0, 0]);
    const VEC_NUM_9: (u8, u8, [u8; 10]) = (5, 10, [0, 0, 232, 197, 225, 17, 0, 0, 0, 0]);
    const VEC_NUM_0: (u8, u8, [u8; 10]) = (5, 10, [0, 0, 232, 198, 46, 0, 0, 0, 0, 0]);
    const VEC_NUM_PERIOD: (u8, u8, [u8; 10]) = (2, 10, [0, 15, 0, 0, 0, 0, 0, 0, 0, 0]);
    const VEC_NUM_MINUS: (u8, u8, [u8; 7]) = (4, 10, [0, 0, 240, 0, 0, 0, 0]);

    struct ProviderClient;
    struct ContainerClient([bool; 300]);

    impl SourceProvider for ProviderClient {
        fn pixels(&self, n: u8) -> &[u8] {
            match n {
                0 => &VEC_NUM_0.2,
                1 => &VEC_NUM_1.2,
                2 => &VEC_NUM_2.2,
                3 => &VEC_NUM_3.2,
                4 => &VEC_NUM_4.2,
                5 => &VEC_NUM_5.2,
                6 => &VEC_NUM_6.2,
                7 => &VEC_NUM_7.2,
                8 => &VEC_NUM_8.2,
                9 => &VEC_NUM_9.2,
                MINUS => &VEC_NUM_MINUS.2,
                PERIOD => &VEC_NUM_PERIOD.2,
                _ => unreachable!(), // unreachable
            }
        }

        fn width(&self, n: u8) -> u8 {
            match n {
                0 => VEC_NUM_0.0,
                1 => VEC_NUM_1.0,
                2 => VEC_NUM_2.0,
                3 => VEC_NUM_3.0,
                4 => VEC_NUM_4.0,
                5 => VEC_NUM_5.0,
                6 => VEC_NUM_6.0,
                7 => VEC_NUM_7.0,
                8 => VEC_NUM_8.0,
                9 => VEC_NUM_9.0,
                MINUS => VEC_NUM_MINUS.0,
                PERIOD => VEC_NUM_PERIOD.0,
                _ => unreachable!(), // unreachable
            }
        }
    }

    fn to_s(canvas_w: usize, canvas_h: usize, data: &[bool]) -> String {
        // \n is for easy assertion
        let mut result = "\n".to_string();
        for row in &data.iter().take(canvas_w * canvas_h).chunks(canvas_w) {
            row.for_each(|b| result += &format!("{}", if *b { "■" } else { "□" }));
            result += "\n";
        }
        result
    }

    impl DataContainer for ContainerClient {
        fn update(&mut self, index: usize, b: bool) {
            self.0[index] = b;
        }

        fn data(&self) -> &[bool] {
            &self.0
        }
    }

    fn numbers() -> MonoImageNumbers<ProviderClient, ContainerClient> {
        MonoImageNumbers::new(10, ProviderClient, ContainerClient([false; 300]))
    }

    #[test]
    fn test_normal() {
        let mut n = numbers();

        let (w, h) = n.update(11185);
        assert_eq!(23, w);
        assert_eq!(10, h);
        assert_eq!(
            r#"
□□□□□□□□□□□□□■■■□□□□□□□
□□□□□□□□□□□□■□□□■□□□□□□
□□□□□□□□□□□□■□□□■□□□□□□
□■□□□■□□□■□□□■■■□□□■■■■
■■□□■■□□■■□□■□□□■□□■□□□
□■□□□■□□□■□□■□□□■□□■■■□
□■□□□■□□□■□□■□□□■□□□□□■
■■■□■■■□■■■□□■■■□□□□□□■
□□□□□□□□□□□□□□□□□□■□□□■
□□□□□□□□□□□□□□□□□□□■■■□
"#,
            to_s(w, h, n.container.data())
        );
    }

    #[test]
    fn test_update() {
        let mut n = numbers();

        n.update(11185);
        let (w, h) = n.update(20);
        assert_eq!(11, w);
        assert_eq!(10, h);
        assert_eq!(
            r#"
□□□□□□□□□□□
□□□□□□□□□□□
□□□□□□□□□□□
□■■■□□□■■■□
■□□□■□■□□□■
□□□■□□■□□□■
□□■□□□■□□□■
■■■■■□□■■■□
□□□□□□□□□□□
□□□□□□□□□□□
"#,
            to_s(w, h, n.container.data())
        );
    }

    #[test]
    fn test_minus() {
        let mut n = numbers();

        let (w, h) = n.update(-119);
        assert_eq!(18, w);
        assert_eq!(10, h);
        assert_eq!(
            r#"
□□□□□□□□□□□□□□□□□□
□□□□□□□□□□□□□□□□□□
□□□□□□□□□□□□□□□□□□
□□□□□□■□□□■□□□■■■□
■■■■□■■□□■■□□■□□□■
□□□□□□■□□□■□□■□□□■
□□□□□□■□□□■□□□■■■■
□□□□□■■■□■■■□□□□□■
□□□□□□□□□□□□□□□□■□
□□□□□□□□□□□□□□□■□□
"#,
            to_s(w, h, n.container.data())
        );
    }

    #[test]
    fn test_float() {
        let mut n = numbers();

        let (w, h) = n.update_f(-19.1234, 2);
        assert_eq!(27, w);
        assert_eq!(10, h);
        assert_eq!(
            r#"
□□□□□□□□□□□□□□□□□□□□□□□□□□□
□□□□□□□□□□□□□□□□□□□□□□□□□□□
□□□□□□□□□□□□□□□□□□□□□□□□□□□
□□□□□□■□□□■■■□□□□□□■□□□■■■□
■■■■□■■□□■□□□■□□□□■■□□■□□□■
□□□□□□■□□■□□□■□□□□□■□□□□□□■
□□□□□□■□□□■■■■□■■□□■□□□□■■□
□□□□□■■■□□□□□■□■■□■■■□□□□□■
□□□□□□□□□□□□■□□□□□□□□□■□□□■
□□□□□□□□□□□■□□□□□□□□□□□■■■□
"#,
            to_s(w, h, n.container.data())
        );
    }
}
