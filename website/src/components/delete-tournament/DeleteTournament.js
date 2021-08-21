import React, { useState } from 'react'
import * as Api from '../../api/TournamentApi'
import Button from '../../components/button/Button'
import Modal from 'react-modal'

function deleteTournament(id) {
  Api.deleteTournament(id)
    .then(() => {
      window.location.href = '/tournaments'
    })
    .catch((e) => (window.location.href = '/'))
}

export default function DeleteTournament(id) {
  const [modalIsOpen, setIsOpen] = useState(false)

  function closeModal() {
    setIsOpen(false)
  }

  return (
    <>
      <div onClick={() => setIsOpen(true)}>
        <Button placeholder="Delete tournament" />
      </div>
      <Modal
        className="Modal"
        overlayClassName="Overlay"
        isOpen={modalIsOpen}
        onRequestClose={closeModal}
        ariaHideApp={false}
      >
        <div className="modal-body">
          <h3>Are you sure you want to delete the tournament?</h3>
          <div onClick={() => deleteTournament(id.id)}>
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
