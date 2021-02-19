/*
    This file contains a semantic error:
    network `name` is defined twice
*/
network Name {
    automata A {
        begin a1
        state a2
        state a3

        trans t1 a1 a2
        trans t2 a2 a3
        trans t3 a3 a1
    }
}

request Name {
    space
}

network Name {
    automata A {
        begin a1
        state a2
        state a3

        trans t1 a1 a2
        trans t2 a2 a3
        trans t3 a3 a1
    }
}