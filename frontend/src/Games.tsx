import React, { Fragment, useEffect, useState, useRef } from 'react';
import './form.css';
import './flex.css';
import './games.css';
import { checkError, GAME_INDEX, postArgs, rejectedPromiseHandler, SessionInfo, GET_GAME, GAME_NEW, GAME_JOIN, GAME_LEAVE, GAME_START } from './api';
import Gomoku from './Gomoku';
import { Link, useParams } from 'react-router-dom';

export const COLORS = ["black", "white"];
export const TEXT_COLORS = ["white", "black"];

interface NewGameProps {
  game_update_callback: () => void
}

function NewGame(props: NewGameProps) {
  const [name, setName] = useState("");

  function submit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if(name === "") return;

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
  const [shedId, setShedId] = useState(-1);

  const mountedRef = useRef(true);

  function loadGames() {
    if(shedId !== -1) {
      window.clearTimeout(shedId);
    }

    fetch(GAME_INDEX, {
      method: 'GET'
    }).then(resp => (resp.json())).then(json => {
      if(checkError(json)) {
        setTotalNumGames(json.games.length);
        const games = json.games.slice(0, numLoaded);
        setGames(games);
      }
    }).catch(rejectedPromiseHandler).finally(() => {
      if(shedId !== -1) {
        window.clearTimeout(shedId);
      }
      if(mountedRef.current) {
        setShedId(window.setTimeout(() => loadGames(), 2000));
      }
    });
  }

  useEffect(() => {
    loadGames();

    return () => {
      mountedRef.current = false;
      if(shedId !== -1) window.clearTimeout(shedId);
    }
  }, []);

  return (
    <div className="gamesContainer">
      {props.session.logged_in &&
        <NewGame game_update_callback={() => loadGames()} />
      }
      {games.map((id) =>
        <Game id={id} session={props.session} game_update_callback={() => loadGames()} key={id} />
      )}
      {(totalNumGames === -1 || numLoaded < totalNumGames + 3) &&
        <button onClick={
          () => { 
            setNumLoaded(numLoaded + 3);
            loadGames();
          }
        } className="btn">Load More Games</button>
      }
      {totalNumGames === 0 &&
        <p>No games found.</p>
      }
    </div>
  );
}

export interface GameProps {
  id: number,
  session: SessionInfo,
  game_update_callback: () => void,
  show_expand?: boolean,
}

export function Game(props: GameProps) {
  const [game, setGame] = useState(null as any);
  const [shedId, setShedId] = useState(-1);

  const mountedRef = useRef(true);

  function loadGame(id: number) {
    if(shedId !== -1) {
      window.clearTimeout(shedId);
    }
    fetch(GET_GAME(id) + "?dont_invert=true", {
      method: 'GET'
    }).then((resp) => resp.json()).then((json) => {
      if(checkError(json)) {
        setGame(json);
      }
    }).catch(rejectedPromiseHandler).finally(() => {
      if(shedId !== -1) {
        window.clearTimeout(shedId);
      }
      if(mountedRef.current) {
        setShedId(window.setTimeout(() => loadGame(id), 1500));
      }
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

  useEffect(() => { 
    loadGame(props.id);
    
    return () => {
      mountedRef.current = false;
      if(shedId !== -1) {
        window.clearTimeout(shedId);
      }
    }
  }, []);

  if(game === null) return (<div></div>);

  let waitingOn = "";
  let waitingOnUs = false;
  let usInGame = false;
  for(let i = 0; i < game.waiting_on.length; i++) {
    if(props.session.logged_in && props.session.id === game.player_ids[i]) {
      usInGame = true;
    }
    if(game.waiting_on[i]) {
      if (waitingOn !== "") waitingOn += ", ";
      waitingOn += game.players[i];

      if(props.session.logged_in && props.session.id === game.player_ids[i]) {
        waitingOnUs = true;
      }
    }
  }

  const game_info = (
    <Fragment>
      <div className="flexShrink infoElem">
        <span>
          {game.started ? (game.active ? "Active" : "Finished") : "Waiting to Start"}
        </span>
      </div>
      <div className="flexShrink infoElem">
        <span>
          {waitingOn !== "" && <span>Waiting On: {waitingOn}</span>}
        </span>
      </div>
      <div className="flexExpand" />
      <div className="flexShrink infoElem">
        <span>
          Outcome: {game.outcome}
        </span>
      </div>
    </Fragment>
  )

  // figure out game action permissions
  const showJoin = !game.player_ids.includes(props.session.id);
  const showLeave = !showJoin;
  const showStart = game.owner_id === props.session.id;

  const action_controls = (props.session.logged_in && !game.started) && (
    <Fragment>
      {showStart && 
        <div className="flexShrink">
          <button className="btn gameActionsBtn" onClick={(e) => game_action(GAME_START)}>Start Game</button>
        </div>
      }
      {showJoin &&
        <div className="flexShrink gameActionsBtn">
          <button className="btn gameActionsBtn" onClick={(e) => game_action(GAME_JOIN)}>+ Join Game</button>
        </div>
      }
      {showLeave &&
        <div className="flexShrink gameActionsBtn">
          <button className="btn gameActionsBtn" onClick={(e) => game_action(GAME_LEAVE)}>- Leave Game</button>
        </div>
      }
    </Fragment>
  );

  return (
    <div className="gameCont">
      <div className="gameTitle">
        <span className="gameTitle">
          <Link to={`/game/${props.id}`}>{game.name} {props.show_expand != false && "⮞"}</Link>
        </span>
        <span className="gameId">
          <code>ID {props.id}</code>
        </span>
      </div>
      <div className="gamePlayers">
        {game.players.length === 0 &&
          <div 
            className="gamePlayer" 
            style={{backgroundColor: "#aaa"}}>
              No Players Yet
          </div>
        }
        {game.players.length !== 0 &&
          game.players.map((player: string, index: number) =>
            <div className="gamePlayer" style={{backgroundColor: COLORS[index], color: TEXT_COLORS[index]}} key={index}>{player}</div>
          )
        }
      </div>
      <div className="flexHorizontal gameActions flexVerticalMobile">
        {game_info}
        {action_controls}
      </div>
      <div className="playInstructions">
        {waitingOnUs && <span>It is your turn. Click where you would like to move.</span>}
        {!waitingOnUs && usInGame && <span>It is not your turn.</span>}
      </div>
      <Gomoku colors={COLORS} id={props.id} state={game.state} width={Math.min(window.innerWidth - 30, 700)} height={Math.min(window.innerWidth - 30, 700)} do_play={waitingOnUs} />
    </div>
  );
}

export interface UrlGameProps {
  session: SessionInfo,
};

interface UrlGameUrlParams {
  game_id: string;
}

export function UrlGame(props: UrlGameProps) {
  const { game_id } = useParams<UrlGameUrlParams>();

  return (
    <Game game_update_callback={() => {}} session={props.session} id={parseInt(game_id, 10)} show_expand={false} />
  )
}