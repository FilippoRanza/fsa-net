
network TestNetwork {
    automata A {
        begin a1
        state a2
        state a3

        trans t1 a1 a2
        trans t2 a2 a3
        trans t3 a3 a1
    }

    automata B {
        begin b1
        state b2
        state b3

        trans t1 b1 b2
        trans t2 b2 b3
        trans t3 b3 b1
    }

}

request TestNetwork {
    space
}


network OtherNetwork {
    automata A {
        begin a1
        state a2
        state a3

        trans t1 a1 a2
        trans t2 a2 a3
        trans t3 a3 a1
    }

    automata B {
        begin b1
        state b2
        state b3

        trans t1 b1 b2
        trans t2 b2 b3
        trans t3 b3 b1
    }
}

request OtherNetwork {
    space
}



