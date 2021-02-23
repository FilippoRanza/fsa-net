
network Test {
    automata A {
        begin s0
        begin s1
        trans t1 s0 s1
    }

    automata B {
        begin s0
        state s1
        trans t1 s0 s1
    }

    link L A B
}

