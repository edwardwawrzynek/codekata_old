import React from 'react';
import { Link } from 'react-router-dom';
import './form.css';

export default function AuthRequired(props: {}) {
  return (
    <div className="formContainer">
      <div className="form">
        You must be signed in to access this page.
        <Link to="/" className="btn">Go Home</Link> 
      </div>
    </div>
  );
}