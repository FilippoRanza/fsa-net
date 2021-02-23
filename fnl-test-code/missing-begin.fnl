network Test {
    automata A {
        state s1
        state s2
        trans t1 s1 s2
    }

    automata B {
        begin s0
        state s1
        trans t1 s0 s1
    }

    link l A B

}