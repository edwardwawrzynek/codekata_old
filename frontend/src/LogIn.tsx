import React, { useState } from 'react';
import './form.css';
import './flex.css';
import { NEW_SESSION, postArgs, rejectedPromiseHandler } from './api';
import { useHistory } from "react-router-dom";

export interface LogInProps {
  callback: () => void
}

export default function LogIn(props: LogInProps) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [errorMsg, setErrorMsg] = useState("");

  let history = useHistory();

  function submit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if(username === "" || password === "") {
      setErrorMsg("email or password can't be empty");
    } else {
      fetch(NEW_SESSION, {
        method: 'POST',
        credentials: 'include',
        body: postArgs({
          username: username,
          password: password,
        }),
      }).then(resp => (resp.json())).then(json => {
        if(json.error !== undefined) {
          setErrorMsg(json.error);
        } else {
          props.callback();
          history.push("/");
        }
      }).catch(rejectedPromiseHandler);
    }
  }

  return (
    <div className="formContainer">
        <form className="form" onSubmit={(e) => { submit(e); }}>
          <span className="formTitle">Codekata Log In</span>
          {errorMsg !== "" &&
            <div className="error">
              {errorMsg}
            </div>
          }
          <label>
            <input type="email" value={username} onChange={(e) => setUsername(e.target.value)} placeholder="Email"></input>
          </label>
          <label>
            <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} placeholder="Password"></input>
          </label>
          <input type="submit" value="Log In" className="btn"></input>
        </form>
      </div>
  );
}
