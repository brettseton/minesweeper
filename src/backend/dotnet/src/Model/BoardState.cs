namespace backend.Model
{
    public enum BoardState : int
    {
        ZERO,
        ONE,
        TWO,
        THREE,
        FOUR,
        FIVE,
        SIX,
        SEVEN,
        EIGHT,
        UNKNOWN = -1,
        MINE = -2,
        FLAG = -3
    }
}