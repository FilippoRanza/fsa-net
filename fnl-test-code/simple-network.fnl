
network TestNetwork {    
    
    link L2 TestB TestA
    link L3 TestA TestB

    events e2, e3
    obs o2, o3
    rel r, f
    automata TestA {
        state b
        begin a
        trans ta {
            src a
            dst b
            input e2(L2)
            output e3(L3)
            obs o2
        }
        trans tb {
            src b
            dst a
            output e3(L3)
            rel r
        }
    }

    automata TestB {
        begin a
        state b
        trans ta {
            src a
            dst b
            output e2(L2)
            obs o3
        }

        trans tb {
            src b 
            dst a
            input e3(L3)
        }

        trans tc {
            src b
            dst b
            input e3(L3)
            rel f
        }

    }



}

request TestNetwork {
    //space
    diagnosis o3, o2
}
