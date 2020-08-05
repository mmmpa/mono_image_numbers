use bit_iterator::BitIterator;
use itertools::Itertools;

pub type NumberVecImage<T> = (usize, usize, Vec<T>);
pub type VecImage = (usize, usize, Vec<bool>);

pub struct Numbers {
    height: usize,
    n0: VecImage,
    n1: VecImage,
    n2: VecImage,
    n3: VecImage,
    n4: VecImage,
    n5: VecImage,
    n6: VecImage,
    n7: VecImage,
    n8: VecImage,
    n9: VecImage,
    period: VecImage,
}

impl Numbers {
    pub fn new(
        height: usize,
        n0: VecImage,
        n1: VecImage,
        n2: VecImage,
        n3: VecImage,
        n4: VecImage,
        n5: VecImage,
        n6: VecImage,
        n7: VecImage,
        n8: VecImage,
        n9: VecImage,
        period: VecImage,
    ) -> Self {
        Self {
            height,
            n0,
            n1,
            n2,
            n3,
            n4,
            n5,
            n6,
            n7,
            n8,
            n9,
            period,
        }
    }

    pub fn from_u8(
        height: usize,
        n0: NumberVecImage<u8>,
        n1: NumberVecImage<u8>,
        n2: NumberVecImage<u8>,
        n3: NumberVecImage<u8>,
        n4: NumberVecImage<u8>,
        n5: NumberVecImage<u8>,
        n6: NumberVecImage<u8>,
        n7: NumberVecImage<u8>,
        n8: NumberVecImage<u8>,
        n9: NumberVecImage<u8>,
        period: NumberVecImage<u8>,
    ) -> Self {
        Self::new(
            height,
            Self::normalize_u8(n0, height),
            Self::normalize_u8(n1, height),
            Self::normalize_u8(n2, height),
            Self::normalize_u8(n3, height),
            Self::normalize_u8(n4, height),
            Self::normalize_u8(n5, height),
            Self::normalize_u8(n6, height),
            Self::normalize_u8(n7, height),
            Self::normalize_u8(n8, height),
            Self::normalize_u8(n9, height),
            Self::normalize_u8(period, height),
        )
    }

    fn normalize_u8(src: NumberVecImage<u8>, height: usize) -> VecImage {
        let (w, h, bit_n) = src;

        let all = bit_n
            .into_iter()
            .flat_map(|n| BitIterator::from(n))
            .take(w * height)
            .collect::<Vec<_>>();

        // #[cfg(test)]
        // Self::tester(w, &all);

        (w, h, all)
    }

    #[cfg(test)]
    fn tester(w: usize, bits: &[bool]) {
        for row in bits.chunks(w) {
            row.iter().for_each(|v| print!("{}", *v as u8));
            print!("\n");
        }
        print!("\n");
    }

    fn img(&self, n: u8) -> &VecImage {
        match n {
            0 => &self.n0,
            1 => &self.n1,
            2 => &self.n2,
            3 => &self.n3,
            4 => &self.n4,
            5 => &self.n5,
            6 => &self.n6,
            7 => &self.n7,
            8 => &self.n8,
            9 => &self.n9,
            _ => &self.n0, // unreachable
        }
    }

    fn w(&self, n: u8) -> usize {
        self.img(n).0
    }

    pub fn generate(&self, n: usize) {
        let mut nums = vec![];
        let mut now = n;
        let mut width = 0;
        while now > 0 {
            let n = (now % 10) as u8;
            width += self.w(n);
            nums.push(n);
            now /= 10
        }

        println!("{:?}", nums);
    }
}

#[cfg(test)]
mod tests {
    use crate::Numbers;

    const VEC_NUM_1: (usize, usize, [u8; 10]) = (3, 10, [0, 44, 151, 0, 0, 0, 0, 0, 0, 0]);
    const VEC_NUM_2: (usize, usize, [u8; 10]) = (5, 10, [0, 0, 232, 136, 159, 0, 0, 0, 0, 0]);
    const VEC_NUM_3: (usize, usize, [u8; 10]) = (5, 10, [0, 0, 232, 132, 193, 139, 128, 0, 0, 0]);
    const VEC_NUM_4: (usize, usize, [u8; 10]) = (5, 10, [0, 0, 35, 42, 95, 16, 128, 0, 0, 0]);
    const VEC_NUM_5: (usize, usize, [u8; 10]) = (5, 10, [0, 0, 244, 56, 33, 139, 128, 0, 0, 0]);
    const VEC_NUM_6: (usize, usize, [u8; 10]) = (5, 10, [34, 33, 232, 198, 46, 0, 0, 0, 0, 0]);
    const VEC_NUM_7: (usize, usize, [u8; 10]) = (5, 10, [0, 1, 248, 200, 68, 33, 0, 0, 0, 0]);
    const VEC_NUM_8: (usize, usize, [u8; 10]) = (5, 10, [116, 98, 232, 198, 46, 0, 0, 0, 0, 0]);
    const VEC_NUM_9: (usize, usize, [u8; 10]) = (5, 10, [0, 0, 232, 197, 225, 17, 0, 0, 0, 0]);
    const VEC_NUM_0: (usize, usize, [u8; 10]) = (5, 10, [0, 0, 232, 198, 46, 0, 0, 0, 0, 0]);
    const VEC_NUM_PERIOD: (usize, usize, [u8; 10]) = (2, 10, [0, 15, 0, 0, 0, 0, 0, 0, 0, 0]);

    fn numbers() -> Numbers {
        Numbers::from_u8(
            10,
            (5, 10, vec![0, 0, 232, 198, 46, 0, 0]),
            (3, 10, vec![0, 44, 151, 0]),
            (5, 10, vec![0, 0, 232, 136, 159, 0, 0]),
            (5, 10, vec![0, 0, 232, 132, 193, 139, 128]),
            (5, 10, vec![0, 0, 35, 42, 95, 16, 128]),
            (5, 10, vec![0, 0, 244, 56, 33, 139, 128]),
            (5, 10, vec![34, 33, 232, 198, 46, 0, 0]),
            (5, 10, vec![0, 1, 248, 200, 68, 33, 0]),
            (5, 10, vec![116, 98, 232, 198, 46, 0, 0]),
            (5, 10, vec![0, 0, 232, 197, 225, 17, 0]),
            (2, 10, vec![0, 15, 0]),
        )
    }

    #[test]
    fn test() {
        let n = numbers();

        n.generate(11145);
    }
}
