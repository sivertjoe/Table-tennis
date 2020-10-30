import { React, lazy } from 'react'
import { Switch, Route } from 'react-router-dom'

const HomePage = lazy(() => import('./pages/HomePage'))
const ProfilePage = lazy(() => import('./pages/ProfilePage'))

const Routes = () => (
  <Switch>
    <Route exact path="/" component={HomePage} />
    <Route exact path="/profile/:id" component={ProfilePage} />
  </Switch>
)

export default Routes
