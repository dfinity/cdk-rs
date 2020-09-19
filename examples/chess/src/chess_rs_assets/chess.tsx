import React, { Component } from "react";
import HumanVsActor from "./integration";

class Chess extends Component {
  render() {
    return (
      <div>
        <div style={boardsContainer}>
          <HumanVsActor />
        </div>
      </div>
    );
  }
}

export default Chess;

const boardsContainer = {
  display: "flex",
  justifyContent: "space-around",
  alignItems: "center",
  flexWrap: undefined, //"wrap",
  width: "100vw",
  marginTop: 30,
  marginBottom: 50
};
