import React, { Fragment, useEffect, useState } from 'react';
import { useHistory, useParams, Link } from 'react-router-dom';
import './Header.css';
import './flex.css';
import { checkError, PAGE_EDIT, PAGE_GET, PAGE_NEW, postArgs, rejectedPromiseHandler, SessionInfo } from './api';
import { NotFound } from './App';
import ReactMarkdown from 'react-markdown/with-html';

export interface PageProps {
  session: SessionInfo,
  newPage: boolean,
}

interface PageState {
  id: number,
  url: string,
  content: string,
}

export default function Page(props: PageProps) {
  let params = useParams<Array<string>>();
  let [loaded, setLoaded] = useState(props.newPage);
  let [found, setFound] = useState(true);
  let [page, setPage] = useState({ id: -1, url: "", content: "" });
  let [editMode, setEditMode] = useState(props.newPage);
  let history = useHistory();

  useEffect(() => {
    if(!props.newPage) {
      fetch(PAGE_GET(params[0]), { 
        credentials: 'include',
        method: 'GET',
      }).then(resp => resp.json()).then(json => {
        if(json.success === false) {
          setLoaded(true);
          setFound(false);
        } else {
          setPage({
            id: json.id,
            url: json.url,
            content: json.content,
          });
          setLoaded(true);
        }
      }).catch(rejectedPromiseHandler);
    }

    return () => {};
  }, [params, props.newPage]);

  function submit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    let path = props.newPage ? PAGE_NEW : PAGE_EDIT;
    let args: {[key: string]: string} = props.newPage ? { url: page.url, content: page.content } : { id: page.id.toString(), url: page.url, content: page.content };

    fetch(path, {
      method: 'POST',
      credentials: 'include',
      body: postArgs(args),
    }).then(resp => resp.json()).then(json => {
      if(checkError(json)) {
        setEditMode(false);
        history.push(`/pages/${page.url}`);
      }
    }).catch(rejectedPromiseHandler);
  }

  if(!loaded) {
    return <div></div>;
  }

  if(!found) {
    return <NotFound></NotFound>;
  }

  if(editMode && (!props.session.logged_in || !props.session.is_admin)) {
    return (
      <div className="formContainer">
        <div className="form">
          <p>You don't have permission to see this page.</p>
          <Link to="/" className="btn">Go Home</Link>
        </div>
      </div>
    );
  }

  return (
    <Fragment>
      <div className="pageContainer">
        {!editMode && props.session.is_admin &&
          <button 
            className="btn" 
            style={{margin: 0}} 
            type="button" 
            onClick={e => setEditMode(!editMode)}
          >
            Edit Page
          </button>
        }
        {!editMode &&
          <ReactMarkdown source={page.content} escapeHtml={false} />
        }
        {editMode && 
          <form onSubmit={(e) => { submit(e); }}>
            <span className="formTitle">Edit Page</span>
            <label>
              <input type="text" value={page.url} onChange={(e) => setPage({ id: page.id, url: e.target.value, content: page.content})} placeholder="Url"></input>
            </label>
            <label>
              <textarea value={page.content} onChange={(e) => setPage({ id: page.id, url: page.url, content: e.target.value})} placeholder="Content"></textarea>
            </label>
            <input type="submit" value="Submit" className="btn"></input>
          </form>
        }
      </div>
    </Fragment>
  );
}
