import React, { Component, Fragment, useState } from 'react';
import { Link, Redirect } from 'react-router-dom';
import './form.css';
import './flex.css';
import { GET_USER, NEW_USER, postArgs, rejectedPromiseHandler } from './api';
import { useHistory } from "react-router-dom";

export interface SignUpProps {}

export default function SignUp(props: SignUpProps) {
  const [username, setUsername] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [password, setPassword] = useState("");
  const [passwordAgain, setPasswordAgain] = useState("");
  const [errorMsg, setErrorMsg] = useState("");

  let history = useHistory();

  function submit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if(password != passwordAgain) {
      setErrorMsg("passwords don't match");
    } else if(username == "" || displayName == "" || password == "") {
      setErrorMsg("Email, Display Name, and Password can't be empty.");
    } else {
      fetch(NEW_USER, {
        method: 'POST',
        body: postArgs({
          username: username,
          password: password,
          display_name: displayName
        }),
      }).then(resp => (resp.json())).then(json => {
        if(json.error != undefined) {
          setErrorMsg(json.error);
        } else {
          history.push("/login");
        }
      }).catch(rejectedPromiseHandler);
    }
  }

  return (
    <div className="formContainer">
        <form className="form" onSubmit={(e) => { submit(e); }}>
          <span className="formTitle">Codekata Sign Up</span>
          {errorMsg != "" &&
            <div className="error">
              {errorMsg}
            </div>
          }
          <label>
            <input type="email" value={username} onChange={(e) => setUsername(e.target.value)} placeholder="Email"></input>
          </label>
          <label>
          <input type="text" value={displayName} onChange={(e) => setDisplayName(e.target.value)} placeholder="Display Name"></input>
          </label>
          <label>
            <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} placeholder="Password"></input>
          </label>
          <label>
            <input type="password" value={passwordAgain} onChange={(e) => setPasswordAgain(e.target.value)} placeholder="Password (Again)"></input>
          </label>
          <input type="submit" value="Sign Up" className="btn"></input>
        </form>
      </div>
  );
}
