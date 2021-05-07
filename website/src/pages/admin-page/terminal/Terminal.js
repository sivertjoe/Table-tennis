import { React, Component } from 'react'
import TerminalComp from 'terminal-in-react'
import * as AdminApi from '../../../api/AdminApi'
import '../../../index.css'

class Terminal extends Component {
  constructor() {
    super()

    const token = localStorage.getItem('token')
    if (token) {
      AdminApi.isAdmin(token)
        .then((isAdmin) => {
          if (isAdmin) {
            this.isAdmin = 1
          } else {
            this.isAdmin = -1
          }
        })
        .catch((error) => console.warn(error.message))
        .finally(() => this.setState({}))
    } else this.isAdmin = -1

    this.sqlExectute = this.sqlExectute.bind(this)
    this.getVariable = this.getVariable.bind(this)
    this.setVariable = this.setVariable.bind(this)
  }
  showMsg = () => 'Hello World'

  sqlExectute(str, print) {
    str.shift()
    const command = str.join(' ')

    AdminApi.executeSql(command).then((res) => print(res))
  }

  getVariable(str, print) {
    str.shift()
    AdminApi.getVariable(str[0]).then((res) => print(res))
  }

  setVariable(str, print) {
    str.shift()
    AdminApi.setVariable(str[0], str[1]).then((res) => print(res))
  }

  render() {
    if (this.isAdmin === 1) {
      return (
        <>
          <h1 className="center">Terminal</h1>
          <div
            style={{
              display: 'flex',
              justifyContent: 'center',
              alignItems: 'center',
              height: '100vh',
            }}
          >
            <TerminalComp
              color="green"
              backgroundColor="black"
              startState="maximised"
              hideTopBar={true}
              barColor="black"
              style={{ fontWeight: 'bold', fontSize: '1em' }}
              commands={{
                sql: (str, print) => this.sqlExectute(str, print),
                getVariable: (str, print) => this.getVariable(str, print),
                setVariable: (str, print) => this.setVariable(str, print),
              }}
              descriptions={{
                color: false,
                show: false,
                clear: false,
                sql:
                  'Exectue a sql command: sql <command>: sql select * from users',
                getVariable:
                  'Get server variable, avaliable are: \nis_season\nseason_length\nuser_conf',
                setVariable:
                  'Set server variable, avaliable are: \nis_season\nseason_length\nuser_conf',
              }}
              msg="Help for help"
            />
          </div>
        </>
      )
    } else if (this.isAdmin === -1)
      return (
        <div>
          <img className="arnold" alt="STOP!!!" src={'../unauth.png'} />
        </div>
      )
    else return <h1 style={{ textAlign: 'center' }}>Loading...</h1>
  }
}

export default Terminal
