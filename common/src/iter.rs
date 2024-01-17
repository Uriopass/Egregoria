pub fn chain<T: TupleITChain>(t: T) -> T::Iter {
    t.chain()
}

pub trait TupleITChain {
    type Item;
    type Iter: Iterator<Item = Self::Item>;

    fn chain(self) -> Self::Iter;
}

impl<Item, A: Iterator<Item = Item>, B: Iterator<Item = Item>> TupleITChain for (A, B) {
    type Item = Item;
    type Iter = std::iter::Chain<A, B>;

    fn chain(self) -> Self::Iter {
        self.0.chain(self.1)
    }
}

impl<Item, A: Iterator<Item = Item>, B: Iterator<Item = Item>, C: Iterator<Item = Item>>
    TupleITChain for (A, B, C)
{
    type Item = Item;
    type Iter = std::iter::Chain<std::iter::Chain<A, B>, C>;

    fn chain(self) -> Self::Iter {
        self.0.chain(self.1).chain(self.2)
    }
}

impl<
        Item,
        A: Iterator<Item = Item>,
        B: Iterator<Item = Item>,
        C: Iterator<Item = Item>,
        D: Iterator<Item = Item>,
    > TupleITChain for (A, B, C, D)
{
    type Item = Item;
    type Iter = std::iter::Chain<std::iter::Chain<std::iter::Chain<A, B>, C>, D>;

    fn chain(self) -> Self::Iter {
        self.0.chain(self.1).chain(self.2).chain(self.3)
    }
}

impl<
        Item,
        A: Iterator<Item = Item>,
        B: Iterator<Item = Item>,
        C: Iterator<Item = Item>,
        D: Iterator<Item = Item>,
        E: Iterator<Item = Item>,
    > TupleITChain for (A, B, C, D, E)
{
    type Item = Item;
    type Iter =
        std::iter::Chain<std::iter::Chain<std::iter::Chain<std::iter::Chain<A, B>, C>, D>, E>;

    fn chain(self) -> Self::Iter {
        self.0
            .chain(self.1)
            .chain(self.2)
            .chain(self.3)
            .chain(self.4)
    }
}
