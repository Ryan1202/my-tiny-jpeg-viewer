pub struct ZigZagScan {
    size: usize,
    x: usize,
    y: usize,
    min: usize,
    max: usize,
    counter: usize,
}

impl ZigZagScan {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            x: 0,
            y: 0,
            min: 0,
            max: 0,
            counter: 0,
        }
    }
}

impl Iterator for ZigZagScan {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        let ret = (self.x, self.y);
        if ret.0 >= self.size || ret.1 >= self.size {
            return None;
        } else {
            let cnt = self.counter;
            let (mut i, mut j) = if cnt % 2 == 0 {
                (self.x, self.y)
            } else {
                (self.y, self.x)
            };

            if j == self.min || i == self.max {
                if self.counter < self.size - 1 {
                    self.max += 1;
                } else {
                    self.min += 1;
                }
                self.counter += 1;
                i = self.max;
                j = self.min;
            } else {
                i += 1;
                j -= 1;
            }

            if cnt % 2 == 0 {
                (self.x, self.y) = (i, j);
            } else {
                (self.y, self.x) = (i, j);
            }
        }
        Some(ret)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_zig_zag_scan_3x3() {
        let mut scan = super::ZigZagScan::new(3);
        // x:0->0,y:0->0
        assert_eq!(scan.next(), Some((0, 0)));
        // x:1->0,y:0->1
        assert_eq!(scan.next(), Some((1, 0)));
        assert_eq!(scan.next(), Some((0, 1)));
        // x:0->2,y:2->0
        assert_eq!(scan.next(), Some((0, 2)));
        assert_eq!(scan.next(), Some((1, 1)));
        assert_eq!(scan.next(), Some((2, 0)));
        // x:2->1,y:1->2
        assert_eq!(scan.next(), Some((2, 1)));
        assert_eq!(scan.next(), Some((1, 2)));
        // x:2->2,y:2->2
        assert_eq!(scan.next(), Some((2, 2)));

        assert_eq!(scan.next(), None);
    }
    #[test]
    fn test_zig_zag_scan_8x8() {
        let mut scan = super::ZigZagScan::new(8);
        // x:0->0,y:0->0
        assert_eq!(scan.next(), Some((0, 0)));
        // x:1->0,y:0->1
        assert_eq!(scan.next(), Some((1, 0)));
        assert_eq!(scan.next(), Some((0, 1)));
        // x:0->2,y:2->0
        assert_eq!(scan.next(), Some((0, 2)));
        assert_eq!(scan.next(), Some((1, 1)));
        assert_eq!(scan.next(), Some((2, 0)));
        // x:3->0,y:0->3
        assert_eq!(scan.next(), Some((3, 0)));
        assert_eq!(scan.next(), Some((2, 1)));
        assert_eq!(scan.next(), Some((1, 2)));
        assert_eq!(scan.next(), Some((0, 3)));
        // x:0->4,y:4->0
        assert_eq!(scan.next(), Some((0, 4)));
        assert_eq!(scan.next(), Some((1, 3)));
        assert_eq!(scan.next(), Some((2, 2)));
        assert_eq!(scan.next(), Some((3, 1)));
        assert_eq!(scan.next(), Some((4, 0)));
        // x:5->0,y:0->5
        assert_eq!(scan.next(), Some((5, 0)));
        assert_eq!(scan.next(), Some((4, 1)));
        assert_eq!(scan.next(), Some((3, 2)));
        assert_eq!(scan.next(), Some((2, 3)));
        assert_eq!(scan.next(), Some((1, 4)));
        assert_eq!(scan.next(), Some((0, 5)));
        // x:0->6,y:6->0
        assert_eq!(scan.next(), Some((0, 6)));
        assert_eq!(scan.next(), Some((1, 5)));
        assert_eq!(scan.next(), Some((2, 4)));
        assert_eq!(scan.next(), Some((3, 3)));
        assert_eq!(scan.next(), Some((4, 2)));
        assert_eq!(scan.next(), Some((5, 1)));
        assert_eq!(scan.next(), Some((6, 0)));
        // x:7->0,y:0->7
        assert_eq!(scan.next(), Some((7, 0)));
        assert_eq!(scan.next(), Some((6, 1)));
        assert_eq!(scan.next(), Some((5, 2)));
        assert_eq!(scan.next(), Some((4, 3)));
        assert_eq!(scan.next(), Some((3, 4)));
        assert_eq!(scan.next(), Some((2, 5)));
        assert_eq!(scan.next(), Some((1, 6)));
        assert_eq!(scan.next(), Some((0, 7)));
        // x:1->7,y:7->1
        assert_eq!(scan.next(), Some((1, 7)));
        assert_eq!(scan.next(), Some((2, 6)));
        assert_eq!(scan.next(), Some((3, 5)));
        assert_eq!(scan.next(), Some((4, 4)));
        assert_eq!(scan.next(), Some((5, 3)));
        assert_eq!(scan.next(), Some((6, 2)));
        assert_eq!(scan.next(), Some((7, 1)));
        // x:7->2,y:2->7
        assert_eq!(scan.next(), Some((7, 2)));
        assert_eq!(scan.next(), Some((6, 3)));
        assert_eq!(scan.next(), Some((5, 4)));
        assert_eq!(scan.next(), Some((4, 5)));
        assert_eq!(scan.next(), Some((3, 6)));
        assert_eq!(scan.next(), Some((2, 7)));
        // x:3->7,y:7->3
        assert_eq!(scan.next(), Some((3, 7)));
        assert_eq!(scan.next(), Some((4, 6)));
        assert_eq!(scan.next(), Some((5, 5)));
        assert_eq!(scan.next(), Some((6, 4)));
        assert_eq!(scan.next(), Some((7, 3)));
        // x:7->4,y:4->7
        assert_eq!(scan.next(), Some((7, 4)));
        assert_eq!(scan.next(), Some((6, 5)));
        assert_eq!(scan.next(), Some((5, 6)));
        assert_eq!(scan.next(), Some((4, 7)));
        // x:5->7,y:7->5
        assert_eq!(scan.next(), Some((5, 7)));
        assert_eq!(scan.next(), Some((6, 6)));
        assert_eq!(scan.next(), Some((7, 5)));
        // x:7->6,y:6->7
        assert_eq!(scan.next(), Some((7, 6)));
        assert_eq!(scan.next(), Some((6, 7)));
        // x:7->7,y:7->7
        assert_eq!(scan.next(), Some((7, 7)));

        assert_eq!(scan.next(), None);
    }
}
