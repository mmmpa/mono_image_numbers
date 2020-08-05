use bit_iterator::BitIterator;
use itertools::Itertools;

pub type VecImage<T> = (u8, u8, T);

pub struct Numbers<T> {
    height: u8,
    n0: VecImage<T>,
    n1: VecImage<T>,
    n2: VecImage<T>,
    n3: VecImage<T>,
    n4: VecImage<T>,
    n5: VecImage<T>,
    n6: VecImage<T>,
    n7: VecImage<T>,
    n8: VecImage<T>,
    n9: VecImage<T>,
    period: VecImage<T>,
}

impl<T> Numbers<T> {
    pub fn new(
        height: u8,
        n0: VecImage<T>,
        n1: VecImage<T>,
        n2: VecImage<T>,
        n3: VecImage<T>,
        n4: VecImage<T>,
        n5: VecImage<T>,
        n6: VecImage<T>,
        n7: VecImage<T>,
        n8: VecImage<T>,
        n9: VecImage<T>,
        period: VecImage<T>,
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

    fn img(&self, n: u8) -> &VecImage<T> {
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

    fn w(&self, n: u8) -> u8 {
        self.img(n).0
    }

    pub fn num_vec(&self, n: usize) -> (usize, [(u8, u8); 16], Vec<bool>) {
        let mut nums = [(0, 0); 16];
        let mut l = 0;
        let mut now = n;
        let mut width = 0;
        while now > 0 {
            let n = (now % 10) as u8;
            let w = self.w(n);
            width += w;
            nums[l] = (n, w);
            l += 1;
            now /= 10
        }

        println!("{:?}", nums.as_ref());

        (l, nums, vec![false; (width * self.height) as usize])
    }
}

impl Numbers<[u8; 10]> {
    fn bits(&self, n: u8) -> &[u8] {
        &self.img(n).2
    }

    pub fn generate(&self, n: usize) {
        let (l, v, img) = self.num_vec(n);
        let offset = 0;
        for (n, w) in v[0..l].iter().rev() {
            let it = self
                .bits(*n)
                .iter()
                .map(|n| *n)
                .flat_map(|n| BitIterator::from(n))
                .take((w * self.height) as usize)
                .chunks(*w as usize)
                .into_iter()
                .for_each(|row| {
                    row.into_iter().for_each(|b| print!("{}", b as u8));
                    print!("\n")
                });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Numbers;

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

    fn numbers() -> Numbers<[u8; 10]> {
        Numbers::new(
            10,
            VEC_NUM_0,
            VEC_NUM_1,
            VEC_NUM_2,
            VEC_NUM_3,
            VEC_NUM_4,
            VEC_NUM_5,
            VEC_NUM_6,
            VEC_NUM_7,
            VEC_NUM_8,
            VEC_NUM_9,
            VEC_NUM_PERIOD,
        )
    }

    #[test]
    fn test() {
        let n = numbers();

        n.generate(11145);
    }
}
