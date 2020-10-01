import React, { Component, Fragment, useEffect, useState, useRef } from 'react';
import './form.css';
import './flex.css';
import { render } from 'react-dom';
import { checkError, GAME_MOVE, postArgs, rejectedPromiseHandler } from './api';

export interface GomokuProps {
  state: any,
  id: number,
  colors: string[],
  width: number,
  height: number,
  do_play: boolean,
}

export default function Gomoku(props: GomokuProps) {
  const isMounted = useRef(true);

  const w = props.width;
  const h = props.height;
  const BOARD_SIZE = 15;
  const CELL_SIZE = w/(BOARD_SIZE + 1);

  function drawGame() {
    const canvas = document.getElementById(`canvas${props.id}`) as HTMLCanvasElement;
    const ctx = canvas.getContext("2d") as CanvasRenderingContext2D;
    
    // clear board
    ctx.fillStyle = "#FFE4C4";
    ctx.fillRect(0, 0, w, h);

    // draw out lines
    for(let x = CELL_SIZE; x <= w - CELL_SIZE/2; x += CELL_SIZE) {
      ctx.beginPath();
      ctx.moveTo(x, CELL_SIZE);
      ctx.lineTo(x, h - CELL_SIZE);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(CELL_SIZE, x);
      ctx.lineTo(w - CELL_SIZE, x);
      ctx.stroke();
    }
    if(props.state == null) {
      return;
    }

    // draw pieces
    for(let x = 0; x < BOARD_SIZE; x++) {
      for(let y = 0; y < BOARD_SIZE; y++) {
        if(props.state.board[x][y] == -1) {
          continue;
        }
        let color = props.colors[props.state.board[x][y]];
        ctx.fillStyle = color;

        ctx.beginPath();
        ctx.arc((x + 1) * CELL_SIZE, (y + 1) * CELL_SIZE, w/(BOARD_SIZE*2.3), 0, 2*Math.PI);
        ctx.closePath();
        ctx.fill();
        ctx.stroke();
      }
    }

  }

  function handleClick(e: React.MouseEvent<HTMLElement>) {
    if(!props.do_play) return;
    // calculate pixel position
    const elem = document.getElementById(`canvas${props.id}`) as HTMLCanvasElement;
    let px = e.pageX - elem.offsetLeft - elem.clientLeft;
    let py = e.pageY - elem.offsetTop - elem.clientTop;

    // calculate cell
    let x = Math.min(Math.max(Math.floor((px / CELL_SIZE) - 0.5), 0), 18);
    let y = Math.min(Math.max(Math.floor((py / CELL_SIZE) - 0.5), 0), 18);

    // make request
    fetch(GAME_MOVE(props.id), {
      method: 'POST',
      credentials: 'include',
      body: postArgs({ x: x.toString(), y: y.toString()})
    }).then(resp => resp.json()).then(json => {
      if(!isMounted.current) return;
      checkError(json);
    }).catch(rejectedPromiseHandler);
  }

  useEffect(() => {
    drawGame();

    return () => {
      isMounted.current = false;
    }
  });

  return (
    <canvas id={`canvas${props.id}`} className="gomoku" width={props.width} height={props.height} onClick={handleClick}></canvas>
  );
}
