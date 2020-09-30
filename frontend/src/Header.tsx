import React, { Component, Fragment } from 'react';
import { Link } from 'react-router-dom';
import './Header.css';
import './flex.css';
import { DELETE_SESSION, GET_USER, rejectedPromiseHandler, SessionInfo } from './api';

export interface HeaderProps {
  session: SessionInfo,
  session_change_callback: () => void
}

export default class Header extends Component<HeaderProps, any> {

  state = {
    user_menu_toggled: false,
  }

  constructor(props: HeaderProps) {
    super(props);
  }

  logged_out_btns() {
    return (
      <Fragment>
        <div className="flexShrink headerBtn">
          <Link to="/signup"><span>Sign Up</span></Link>
        </div>
        <div className="flexShrink headerBtn">
          <Link to="/login"><span>Log In</span></Link>
        </div>
      </Fragment>
    )
  }

  logout = () => {
    this.setState({user_menu_toggled: false});
    fetch(DELETE_SESSION, {
      method: 'POST',
      credentials: 'include'
    }).then(resp => {
      this.props.session_change_callback();
    }).catch(rejectedPromiseHandler);
  }

  logged_in_btns() {
    return (
      <div className="flexShrink headerBtn">
        <span onClick={
          () => this.setState(
            (state: any, props: HeaderProps) => {
              return {user_menu_toggled: !state.user_menu_toggled}})
        }>
          {this.props.session.display_name} &#9660;
        </span>
        {this.state.user_menu_toggled &&
          <div className="userMenuDropdown">
            <div>
              <Link to="/gen_api" onClick={
                () => {this.setState({user_menu_toggled: false})}
              }>Generate API Key</Link>
            </div>
            <div>
              <Link to="/user_edit" onClick={
                  () => {this.setState({user_menu_toggled: false})}
                }>Edit User</Link>
            </div>
            <div>
              <Link to="/" onClick={this.logout}>Log Out</Link>
            </div>
          </div>
        }
      </div>
    )
  }

  header_btns() {
    return this.props.session.logged_in ? this.logged_in_btns() : this.logged_out_btns()
  }

  render() {
    return (
      <div className="header">
        <div className="flexHorizontal">
          <div className="flexShrink">
            <Link to=""><span>Codekata</span></Link>
          </div>
          <div className="flexExpand" />
          { this.header_btns() }
        </div>
      </div>
    );
  }
}
