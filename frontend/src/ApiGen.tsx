import React, { Component, Fragment, useState } from 'react';
import { Link, Redirect } from 'react-router-dom';
import './form.css';
import './flex.css';
import { checkError, GEN_API_KEY, GET_USER, NEW_SESSION, NEW_USER, postArgs, rejectedPromiseHandler, SessionInfo } from './api';
import { useHistory } from "react-router-dom";
import AuthRequired from './AuthRequired';

export interface ApiGenProps {
  session: SessionInfo;
  session_change_callback: () => void;
}

export default function ApiGen(props: ApiGenProps) {
  const [generated, setGenerated] = useState(false);
  const [key, setKey] = useState("");

  let history = useHistory();

  if(!props.session.logged_in) {
    return <AuthRequired />;
  }

  function generate() {
    fetch(GEN_API_KEY, {
      method: 'POST',
      credentials: 'include'
    }).then((resp) => resp.json()).then(json => {
      if(checkError(json)) {
        setGenerated(true);
        setKey(json.key);
      }
      props.session_change_callback();
    }).catch(rejectedPromiseHandler);
  }

  return (
    <div className="formContainer">
      <div className="form">
          <span className="formTitle">API Key Generation</span>
            <p>
              API keys can be used to interact programmatically with Codekata. API keys should be included as the <code>X-API-KEY</code> http header for each call. Documentation for the API calls can be found TODO.
            </p>

            {props.session.has_api_key && !generated && 
              <p>Your account already has an API key associated with it. By generating a new one, your old API key will become invalid. </p> 
            }

          {!generated &&
            <button className="btn" onClick={generate}>{props.session.has_api_key ? "Regenerate API Key" : "Generate API Key"}</button>
          }
          {generated &&
            <Fragment>
              <p>Your API key is:</p>
              <code>{key}</code>
              <p>Once you leave this page, you won't be able to retrieve your API key again. If you lose it, you'll need to generate a new one.</p>
              <Link to="/" className="btn">Go Home</Link>
            </Fragment>
          }
      </div>
    </div>
  );
}
