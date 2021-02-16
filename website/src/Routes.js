import { React, lazy } from 'react'
import { Switch, Route, Redirect } from 'react-router-dom'

const HomePage = lazy(() => import('./pages/home-page/HomePage'))
const RegisterPage = lazy(() => import('./pages/register/RegisterPage'))
const ProfilePage = lazy(() => import('./pages/profile-page/ProfilePage'))
const RegisterMatch = lazy(() => import('./pages/register-match/RegisterMatch'))
const History = lazy(() => import('./pages/history/History'))
const LoginPage = lazy(() => import('./pages/login-page/LoginPage'))
const ChangePasswordPage = lazy(() =>
  import('./pages/change-password-page/ChangePasswordPage'),
)
const Admin = lazy(() => import('./pages/admin-page/Admin'))
const EditMatches = lazy(() =>
  import('./pages/admin-page/edit-matches/EditMatches.js'),
)
const EditSeason = lazy(() =>
  import('./pages/admin-page/edit-season/EditSeason.js'),
)
const StatsPage = lazy(() => import('./pages/stats-page/StatsPage.js'))
const Terminal = lazy(() => import('./pages/admin-page/terminal/Terminal.js'))

const Routes = () => (
  <Switch>
    <Route exact path="/" component={HomePage} />
    <Route exact path="/register" component={RegisterPage} />
    <Route exact path="/profiles" component={ProfilePage} />
    <Route exact path="/profiles/:user" component={ProfilePage} />
    <Route exact path="/match" component={RegisterMatch} />
    <Route exact path="/history" component={History} />
    <Route exact path="/login" component={LoginPage} />
    <Route exact path="/change-password" component={ChangePasswordPage} />
    <Route exact path="/admin" component={Admin} />
    <Route exact path="/admin/edit-match" component={EditMatches} />
    <Route exact path="/admin/edit-season" component={EditSeason} />
    <Route exact path="/stats" component={StatsPage} />
    <Route exact path="/admin/terminal" component={Terminal} />
    <Redirect to="/" />
  </Switch>
)

export default Routes
