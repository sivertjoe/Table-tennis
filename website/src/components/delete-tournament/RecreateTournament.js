import React, { useState } from 'react'
import * as Api from '../../api/TournamentApi'
import Button from '../../components/button/Button'
import Modal from 'react-modal'
function recreateTournament(id) {
  Api.recreateTournament(id)
    .then((tid) => {
      window.location.href = '/tournaments?match=' + tid
    })
    .catch((e) => (window.location.href = '/'))
}

export default function RecreateTournament(id) {
  const [modalIsOpen, setIsOpen] = useState(false)

  function closeModal() {
    setIsOpen(false)
  }

  return (
    <>
      <div onClick={() => setIsOpen(true)}>
        <Button placeholder="Recreate tournament" />
      </div>
      <Modal
        className="Modal"
        overlayClassName="Overlay"
        isOpen={modalIsOpen}
        onRequestClose={closeModal}
        ariaHideApp={false}
      >
        <div className="modal-body">
          <h3>Are you sure you want to rerun the tournament?</h3>
          <div onClick={() => recreateTournament(id.id)}>
            <Button style={{ marginTop: '2rem' }} placeholder="Yes" />
          </div>
          <div onClick={closeModal}>
            <Button style={{ marginTop: '2rem' }} placeholder="No" />
          </div>
        </div>
      </Modal>
    </>
  )
}
