import { React, lazy } from 'react'
import { Switch, Route } from 'react-router-dom'

const HomePage = lazy(() => import('./pages/home-page/HomePage'))
const ProfilePage = lazy(() => import('./pages/profile-page/ProfilePage'))

const Routes = () => (
  <Switch>
    <Route exact path="/" component={HomePage} />
    <Route exact path="/profiles" component={ProfilePage} />
    <Route exact path="/profiles/:user" component={ProfilePage} />
  </Switch>
)

export default Routes
