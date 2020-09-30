import React, { Component, Fragment, useEffect, useState } from 'react';
import { Link, Redirect } from 'react-router-dom';
import './form.css';
import './flex.css';
import './games.css';
import { checkError, GAME_INDEX, GET_USER, NEW_SESSION, NEW_USER, postArgs, rejectedPromiseHandler, SessionInfo, USER_EDIT, GET_GAME, GAME_NEW, GAME_JOIN, GAME_LEAVE, GAME_START } from './api';
import { useHistory } from "react-router-dom";
import AuthRequired from './AuthRequired';

export const COLORS = ["CornflowerBlue", "Crimson", "DarkOrange", "Olive"];

interface NewGameProps {
  game_update_callback: () => void
}

function NewGame(props: NewGameProps) {
  const [name, setName] = useState("");

  function submit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if(name == "") return;

    fetch(GAME_NEW, {
      method: 'POST',
      credentials:'include',
      body: postArgs({ name: name }),
    }).then(resp => resp.json()).then(json => {
      if(checkError(json)) {
        setName("");
        props.game_update_callback();
      }
    }).catch(rejectedPromiseHandler);
  }

  return (
    <form className="newGameForm" onSubmit={submit}>
      <div className="flexShrink newGameTitle">New Game:</div>
      <div className="flexExpand">
        <input type="text" value={name} onChange={(e) => setName(e.target.value)} placeholder="Game Name"></input>
      </div>
      <div className="flexShrink newGameSubmit">
        <input type="submit" value="Create" className="btn newGameBtn"></input>
      </div>
    </form>
  )
}

export interface GamesProps {
  session: SessionInfo,
}

export default function Games(props: GamesProps) {
  const [numLoaded, setNumLoaded] = useState(3);
  const [totalNumGames, setTotalNumGames] = useState(-1);
  const [games, setGames] = useState([] as number[]);

  let history = useHistory();

  function loadGames(numToLoad: number) {
    fetch(GAME_INDEX, {
      method: 'GET'
    }).then(resp => (resp.json())).then(json => {
      if(checkError(json)) {
        setTotalNumGames(json.games.length);
        const games = json.games.slice(0, numToLoad);
        setGames(games);
      }
    }).catch(rejectedPromiseHandler);
  }

  useEffect(() => loadGames(numLoaded), []);

  return (
    <div className="gamesContainer">
      {props.session.logged_in &&
        <NewGame game_update_callback={() => loadGames(numLoaded)} />
      }
      {games.map((id) =>
        <Game id={id} session={props.session} game_update_callback={() => loadGames(numLoaded)} key={id} />
      )}
      {(totalNumGames == -1 || numLoaded < totalNumGames) &&
        <button onClick={
          () => { 
            loadGames(numLoaded + 3);
            setNumLoaded(numLoaded + 3);
          }
        } className="btn">Load More Games</button>
      }
      {totalNumGames == 0 &&
        <p>No games found.</p>
      }
    </div>
  );
}

export interface GameProps {
  id: number,
  session: SessionInfo,
  game_update_callback: () => void,
}

export function Game(props: GameProps) {
  const [game, setGame] = useState(null as any);

  function loadGame(id: number) {
    fetch(GET_GAME(id), {
      method: 'GET'
    }).then((resp) => resp.json()).then((json) => {
      setGame(json);
      console.log(json);
    })
  }

  function game_action(route: (path: number) => string) {
    fetch(route(props.id), {
      method: 'POST',
      credentials: 'include',
    }).then(resp => resp.json()).then(json => {
      if(checkError(json)) {
        loadGame(props.id);
      }
    }).catch(rejectedPromiseHandler);
  }

  useEffect(() => { loadGame(props.id) }, []);

  if(game == null) return <div></div>;

  // figure out game action permissions
  let showJoin = !game.player_ids.includes(props.session.id);
  let showLeave = !showJoin;
  let showStart = game.owner_id == props.session.id;

  let action_controls = (props.session.logged_in && !game.started) && (
    <Fragment>
      {showStart && 
        <div className="flexShrink">
          <button className="btn gameActionsBtn" onClick={(e) => game_action(GAME_START)}>Start Game</button>
        </div>
      }
      {showJoin &&
        <div className="flexShrink gameActionsBtn">
          <button className="btn" onClick={(e) => game_action(GAME_JOIN)}>+ Join Game</button>
        </div>
      }
      {showLeave &&
        <div className="flexShrink gameActionsBtn">
          <button className="btn" onClick={(e) => game_action(GAME_LEAVE)}>- Leave Game</button>
        </div>
      }
    </Fragment>
  );

  return (
    <div className="gameCont">
      <div className="gameTitle">
        <span className="gameTitle">
          {game.name}
        </span>
        <span className="gameId">
          <code>ID {props.id}</code>
        </span>
      </div>
      <div className="gamePlayers">
        {game.players.length == 0 &&
          <div 
            className="gamePlayer" 
            style={{backgroundColor: "#aaa"}}>
              No Players Yet
          </div>
        }
        {game.players.length != 0 &&
          game.players.map((player: string, index: number) =>
            <div className="gamePlayer" style={{backgroundColor: COLORS[index]}} key={index}>{player}</div>
          )
        }
      </div>
      <div className="flexHorizontal gameActions">
        <div className="flexExpand">
          <span>
            {game.started ? (game.active ? "Active" : "Finished") : "Waiting to Start"}
          </span>
        </div>
        {action_controls}
      </div>
      <div style={{width:"400px", height:"400px", backgroundColor:"red", margin: "0 auto"}}></div>
    </div>
  );
}
