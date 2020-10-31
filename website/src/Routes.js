import { React, lazy } from 'react'
import { Switch, Route } from 'react-router-dom'

const HomePage = lazy(() => import('./pages/home-page/HomePage'))
const RegisterPage = lazy(() => import('./pages/register/RegisterPage'))
const ProfilePage = lazy(() => import('./pages/profile-page/ProfilePage'))
const RegisterMatch = lazy(() => import('./pages/register-match/RegisterMatch'))

const Routes = () => (
  <Switch>
    <Route exact path="/" component={HomePage} />
    <Route exact path="/register" component={RegisterPage} />
    <Route exact path="/profiles" component={ProfilePage} />
    <Route exact path="/profiles/:user" component={ProfilePage} />
    <Route exact path="/match" component={RegisterMatch} />
  </Switch>
)

export default Routes
