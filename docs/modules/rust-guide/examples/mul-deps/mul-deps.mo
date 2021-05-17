actor Multiply {

    var cell : Nat = 1;

    public func mul(n:Nat) : async Nat { cell *= n*3; cell };

    public query func read() : async Nat {
        cell
    };
}

