import React, { ChangeEvent, Component } from "react";
import PropTypes from "prop-types";
import Chess, { ChessInstance } from "chess.js"; // import Chess from  "chess.js"(default) if recieving an error about new Chess() not being a constructor
import chessActor from "ic:canisters/chess_rs";

import Chessboard from "chessboardjsx";

type Props = { children: (...args: any[]) => any };
interface State {
  name: string,
  pieceSquare: any,
  history: any,
  squareStyles: any,
  fen: string,
  square: string,
  dropSquareStyle: any,
  disabled: boolean,
}

class HumanVsActor extends Component<Props, State> {
  static propTypes = { children: PropTypes.func };

  private game: ChessInstance | undefined = undefined;

  state = {
    name: "default",
    disabled: false,

    fen: "start",
    // square styles for active drop square
    dropSquareStyle: {},
    // custom square styles
    squareStyles: {},
    // square with the currently clicked piece
    pieceSquare: "",
    // currently clicked square
    square: "",
    // array of past game moves
    history: []
  };

  componentDidMount() {
    this.game = new (Chess as any)();
    this.reload();
  }

  // keep clicked square style and remove hint squares
  removeHighlightSquare = () => {
    this.setState(({ pieceSquare, history }) => ({
      squareStyles: squareStyling({ pieceSquare, history })
    }));
  };

  // show possible moves
  highlightSquare = (sourceSquare: any, squaresToHighlight: any) => {
    const highlightStyles = [sourceSquare, ...squaresToHighlight].reduce(
      (a, c) => {
        return {
          ...a,
          ...{
            [c]: {
              background:
                "radial-gradient(circle, #fffc00 36%, transparent 40%)",
              borderRadius: "50%"
            }
          },
          ...squareStyling({
            history: this.state.history,
            pieceSquare: this.state.pieceSquare
          })
        };
      },
      {}
    );

    this.setState(({ squareStyles }) => ({
      squareStyles: { ...squareStyles, ...highlightStyles }
    }));
  };

  doMove = ({ sourceSquare, targetSquare }: any) => {
    // see if the move is legal
    let move = this.game!.move({
      from: sourceSquare,
      to: targetSquare,
      promotion: "q" // always promote to a queen for example simplicity
    });

    // illegal move
    if (move === null) return;

    // Block any moves until we save on the canister.
    this.setState(({}) => ({
      disabled: true,
    }));

    let uci = move.from + move.to + (move.promotion || "");

    chessActor.move(this.state.name, uci).then((valid: boolean) => {
      if (!valid) {
        this.setState(({}) => ({ disabled: false }));
        return;
      }

      chessActor.getFen().then(([fen]) => {
        this.setState(({ history, pieceSquare }) => ({
          fen,
          history: this.game!.history({ verbose: true }),
          squareStyles: squareStyling({ pieceSquare, history })
        }));
      });
    });
  };

  onDrop = ({ sourceSquare, targetSquare }: any) => {
    this.doMove({ sourceSquare, targetSquare });
  };

  onMouseOverSquare = (square: any) => {
    // get list of possible moves for this square
    let moves = this.game!.moves({
      square: square,
      verbose: true
    });

    // exit if there are no moves available for this square
    if (moves.length === 0) return;

    let squaresToHighlight = [];
    for (var i = 0; i < moves.length; i++) {
      squaresToHighlight.push(moves[i].to);
    }

    this.highlightSquare(square, squaresToHighlight);
  };

  onMouseOutSquare = (square: any) => this.removeHighlightSquare();

  // central squares get diff dropSquareStyles
  onDragOverSquare = (square: any) => {
    this.setState({
      dropSquareStyle:
        square === "e4" || square === "d4" || square === "e5" || square === "d5"
          ? { backgroundColor: "cornFlowerBlue" }
          : { boxShadow: "inset 0 0 1px 4px rgb(255, 255, 0)" }
    });
  };

  onSquareClick = (square: any) => {
    this.setState(({ history }) => ({
      squareStyles: squareStyling({ pieceSquare: square, history }),
      pieceSquare: square
    }));

    this.doMove({ sourceSquare: this.state.pieceSquare as any, targetSquare: square });
  };

  onSquareRightClick = (square: any) =>
    this.setState({
      squareStyles: { [square]: { backgroundColor: "deepPink" } }
    });

  reload = () => {
    this.setState({ disabled: true });

    chessActor.getState(this.state.name).then(([board]: any) => {
      this.game!.load(board?.fen || "start");
      this.setState({
        disabled: false,
        fen: board?.fen || "start",
        history: this.game!.history({ verbose: true }),
      });
    });
  };

  ai = () => {
    this.setState({ disabled: true });
    chessActor.generateMove(this.state.name).then(() => this.reload());
  };

  reset = () => {
    this.setState({ disabled: true });
    chessActor.new(this.state.name).then(() => this.reload());
  };

  changeName = (ev: ChangeEvent) => {
    this.setState({ disabled: true, name: (ev.target as any).value });
    this.reload();
  };

  render() {
    const { fen, dropSquareStyle, squareStyles } = this.state;

    return this.props.children({
      squareStyles,
      position: fen,
      ai: this.ai,
      name: this.state.name,
      changeName: this.changeName,
      reload: this.reload,
      reset: this.reset,
      onMouseOverSquare: this.onMouseOverSquare,
      onMouseOutSquare: this.onMouseOutSquare,
      onDrop: this.onDrop,
      dropSquareStyle,
      onDragOverSquare: this.onDragOverSquare,
      onSquareClick: this.onSquareClick,
      onSquareRightClick: this.onSquareRightClick
    });
  }
}

export default function WithMoveValidation() {
  return (
    <div>
      <HumanVsActor>
        {({
            position,
            ai,
            name,
            changeName,
            reset,
            reload,
            onDrop,
            onMouseOverSquare,
            onMouseOutSquare,
            squareStyles,
            disabled,
            dropSquareStyle,
            onDragOverSquare,
            onSquareClick,
            onSquareRightClick
          }) => (
            <div>
              <div>Game: <input disabled={disabled} value={name} onBlur={changeName} onChange={changeName} /></div>
              <button disabled={disabled} onClick={reset}>reset</button>
              <button disabled={disabled} onClick={reload}>reload</button>
              <button disabled={disabled} onClick={ai}>ai move</button>

              <Chessboard
                id="board"
                width={320}
                position={position}
                draggable={!disabled}
                onDrop={onDrop}
                onMouseOverSquare={onMouseOverSquare}
                onMouseOutSquare={onMouseOutSquare}
                boardStyle={{
                  borderRadius: "5px",
                  boxShadow: `0 5px 15px rgba(0, 0, 0, 0.5)`
                }}
                squareStyles={squareStyles}
                dropSquareStyle={dropSquareStyle}
                onDragOverSquare={onDragOverSquare}
                onSquareClick={onSquareClick}
                onSquareRightClick={onSquareRightClick}
              />
            </div>
        )}
      </HumanVsActor>
    </div>
  );
}

const squareStyling = ({ pieceSquare, history }: any) => {
  const sourceSquare = history.length && history[history.length - 1].from;
  const targetSquare = history.length && history[history.length - 1].to;

  return {
    [pieceSquare]: { backgroundColor: "rgba(255, 255, 0, 0.4)" },
    ...(history.length && {
      [sourceSquare]: {
        backgroundColor: "rgba(255, 255, 0, 0.4)"
      }
    }),
    ...(history.length && {
      [targetSquare]: {
        backgroundColor: "rgba(255, 255, 0, 0.4)"
      }
    })
  };
};
