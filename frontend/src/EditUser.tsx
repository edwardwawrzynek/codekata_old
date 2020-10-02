import React, { useState } from 'react';
import './form.css';
import './flex.css';
import { rejectedPromiseHandler, SessionInfo, USER_EDIT } from './api';
import { useHistory } from "react-router-dom";
import AuthRequired from './AuthRequired';

export interface EditUser {
  session_change_callback: () => void,
  session: SessionInfo,
}

export default function EditUser(props: EditUser) {
  const [username, setUsername] = useState(props.session.username);
  const [displayName, setDisplayName] = useState(props.session.display_name);
  const [password, setPassword] = useState("");
  const [passwordAgain, setPasswordAgain] = useState("");
  const [errorMsg, setErrorMsg] = useState("");

  let history = useHistory();

  if(!props.session.logged_in) {
    return <AuthRequired />;
  }

  function submit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if(password !== passwordAgain) {
      setErrorMsg("password's don't match");
      return;
    }

    const data = new URLSearchParams();
    if(username !== "") {
      data.append("username", username);
    }
    if(displayName !== "") {
      data.append("display_name", displayName);
    }
    if(password !== "") {
      data.append("password", password);
    }

    fetch(USER_EDIT, {
      method: 'POST',
      credentials: 'include',
      body: data,
    }).then(resp => (resp.json())).then(json => {
      if(json.error !== undefined) {
        setErrorMsg(json.error);
      } else {
        props.session_change_callback();
        if(username !== props.session.username || password !== "") {
          history.push("/login");
        } else {
          history.push("/");
        }
      }
    }).catch(rejectedPromiseHandler);
  }

  return (
    <div className="formContainer">
        <form className="form" onSubmit={(e) => { submit(e); }}>
          <span className="formTitle">Edit User</span>
          {errorMsg !== "" &&
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
            <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} placeholder="New Password"></input>
          </label>
          <label>
            <input type="password" value={passwordAgain} onChange={(e) => setPasswordAgain(e.target.value)} placeholder="New Password (Again)"></input>
          </label>
          <input type="submit" value="Update User" className="btn"></input>
        </form>
      </div>
  );
}
