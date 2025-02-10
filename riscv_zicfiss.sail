/*=======================================================================================*/
/*  RISCV Sail Model                                                                     */
/*                                                                                       */
/*  This Sail RISC-V architecture model, comprising all files and                        */
/*  directories except for the snapshots of the Lem and Sail libraries                   */
/*  in the prover_snapshots directory (which include copies of their                     */
/*  licences), is subject to the BSD two-clause licence below.                           */
/*                                                                                       */
/*  Copyright (c) 2017-2023                                                              */
/*    Ved Shanbhogue                                                                     */
/*    Prashanth Mundkur                                                                  */
/*    Rishiyur S. Nikhil and Bluespec, Inc.                                              */
/*    Jon French                                                                         */
/*    Brian Campbell                                                                     */
/*    Robert Norton-Wright                                                               */
/*    Alasdair Armstrong                                                                 */
/*    Thomas Bauereiss                                                                   */
/*    Shaked Flur                                                                        */
/*    Christopher Pulte                                                                  */
/*    Peter Sewell                                                                       */
/*    Alexander Richardson                                                               */
/*    Hesham Almatary                                                                    */
/*    Jessica Clarke                                                                     */
/*    Microsoft, for contributions by Robert Norton-Wright and Nathaniel Wesley Filardo  */
/*    Peter Rugg                                                                         */
/*    Aril Computer Corp., for contributions by Scott Johnson                            */
/*    Philipp Tomsich                                                                    */
/*    VRULL GmbH, for contributions by its employees                                     */
/*                                                                                       */
/*  All rights reserved.                                                                 */
/*                                                                                       */
/*  This software was developed by the above within the Rigorous                         */
/*  Engineering of Mainstream Systems (REMS) project, partly funded by                   */
/*  EPSRC grant EP/K008528/1, at the Universities of Cambridge and                       */
/*  Edinburgh.                                                                           */
/*                                                                                       */
/*  This software was developed by SRI International and the University of               */
/*  Cambridge Computer Laboratory (Department of Computer Science and                    */
/*  Technology) under DARPA/AFRL contract FA8650-18-C-7809 ("CIFV"), and                 */
/*  under DARPA contract HR0011-18-C-0016 ("ECATS") as part of the DARPA                 */
/*  SSITH research programme.                                                            */
/*                                                                                       */
/*  This project has received funding from the European Research Council                 */
/*  (ERC) under the European Union’s Horizon 2020 research and innovation                */
/*  programme (grant agreement 789108, ELVER).                                           */
/*                                                                                       */
/*                                                                                       */
/*  Redistribution and use in source and binary forms, with or without                   */
/*  modification, are permitted provided that the following conditions                   */
/*  are met:                                                                             */
/*  1. Redistributions of source code must retain the above copyright                    */
/*     notice, this list of conditions and the following disclaimer.                     */
/*  2. Redistributions in binary form must reproduce the above copyright                 */
/*     notice, this list of conditions and the following disclaimer in                   */
/*     the documentation and/or other materials provided with the                        */
/*     distribution.                                                                     */
/*                                                                                       */
/*  THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS''                   */
/*  AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED                    */
/*  TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A                      */
/*  PARTICULAR PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR                  */
/*  CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,                         */
/*  SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT                     */
/*  LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF                     */
/*  USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND                  */
/*  ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,                   */
/*  OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT                   */
/*  OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF                   */
/*  SUCH DAMAGE.                                                                         */
/*=======================================================================================*/

/* ****************************************************************** */
/* copy from riscv_insts_base.sail                                    */
/* ****************************************************************** */

val maybe_aq : bool <-> string
mapping maybe_aq = {
  true  <-> ".aq",
  false <-> ""
}

val maybe_rl : bool <-> string
mapping maybe_rl = {
  true  <-> ".rl",
  false <-> ""
}

/* ****************************************************************** */
/* This file specifies the instructions in the 'Zicfiss' extension.   */
/* ****************************************************************** */
/* Backward-edge CFI: Shadow stack */


/* ****************************************************************** */
/* SSPUSH and C.SSPUSH operation
 *   if (xSSE == 1)
 *     mem[ssp - (XLEN/8)] = X(src)
 *     ssp = ssp - (XLEN/8)
 *   endif
 */
/* ****************************************************************** */
union clause ast = SSPUSH     : (regidx)
mapping clause encdec = SSPUSH(rs2)
      if extensionEnabled(Ext_Zicfiss) & (rs2 == 0b00001 | rs2 == 0b00101)
  <-> 0b1100111 @ rs2 @ 0b00000 @ 0b100 @ 0b00000 @ 0b1110011
      if extensionEnabled(Ext_Zicfiss) & (rs2 == 0b00001 | rs2 == 0b00101)
mapping clause assembly = SSPUSH(rs2)
      if extensionEnabled(Ext_Zicfiss) & (rs2 == 0b00001 | rs2 == 0b00101)
  <-> "sspush" ^ spc() ^ reg_name(rs2)
      if extensionEnabled(Ext_Zicfiss) & (rs2 == 0b00001 | rs2 == 0b00101)
union clause ast = C_SSPUSH   : (regidx)
mapping clause encdec_compressed = C_SSPUSH(0b00001) if extensionEnabled(Ext_Zicfiss)
  <-> 0b0110000010000001 if extensionEnabled(Ext_Zicfiss)
mapping clause assembly = C_SSPUSH(rs2)
      if extensionEnabled(Ext_Zicfiss)
  <-> "c.sspush" ^ spc() ^ reg_name(rs2)
      if extensionEnabled(Ext_Zicfiss)
/* ****************************************************************** */

/* ****************************************************************** */
/* SSPOPCHK and C.SSPOPCHK operation
 *   if (xSSE == 1)
 *     temp = mem[ssp]
 *     if temp != X(src) then software-check exception
 *     else ssp = ssp + (XLEN/8)
 *   endif
 * SSPOPCHK requires write permission as shadow stacks are encoded as
 * pte.xwr = 010b
 */
/* ****************************************************************** */
union clause ast = SSPOPCHK   : (regidx)
mapping clause encdec = SSPOPCHK(rs1)
      if extensionEnabled(Ext_Zicfiss) & (rs1 == 0b00001 | rs1 == 0b00101)
  <-> 0b110011011100 @ rs1 @ 0b100 @ 0b00000 @ 0b1110011
      if extensionEnabled(Ext_Zicfiss) & (rs1 == 0b00001 | rs1 == 0b00101)
mapping clause assembly = SSPOPCHK(rs1)
      if extensionEnabled(Ext_Zicfiss) & (rs1 == 0b00001 | rs1 == 0b00101)
  <-> "sspopchk" ^ spc() ^ reg_name(rs1)
      if extensionEnabled(Ext_Zicfiss) & (rs1 == 0b00001 | rs1 == 0b00101)
union clause ast = C_SSPOPCHK : (regidx)
mapping clause encdec_compressed = C_SSPOPCHK(0b00101)
      if extensionEnabled(Ext_Zicfiss)
  <-> 0b0110001010000001
      if extensionEnabled(Ext_Zicfiss)
mapping clause assembly = C_SSPOPCHK(rs1)
      if extensionEnabled(Ext_Zicfiss)
  <-> "c.sspopchk" ^ spc() ^ reg_name(rs1)
      if extensionEnabled(Ext_Zicfiss)
/* ****************************************************************** */

/* ****************************************************************** */
/* SSRDP operation:
 * X(rd) = (xSSE == 1) ? ssp : 0
 */
/* ****************************************************************** */
union clause ast = SSRDP      : (regidx)
mapping clause encdec = SSRDP(rd)
      if extensionEnabled(Ext_Zicfiss) & rd != zreg
  <-> 0b110011011100 @ 0b00000 @ 0b100 @ rd @ 0b1110011
      if extensionEnabled(Ext_Zicfiss) & rd != zreg
mapping clause assembly = SSRDP(rd)
      if extensionEnabled(Ext_Zicfiss) & rd != zreg
  <-> "ssrdp" ^ spc() ^ reg_name(rd)
      if extensionEnabled(Ext_Zicfiss) & rd != zreg
/* ****************************************************************** */

/* ****************************************************************** */
/* SSAMOSWAP operation:
 *   if privilege_mode != M && menvcfg.SSE == 0
 *     raise illegal-instruction exception
 *   if S-mode is not implemented
 *     raise illegal-instruction exception
 *   else if privilege_mode == U && senvcfg.SSE == 0
 *     raise illegal-instruction exception
 *   else if privilege_mode == VS && henvcfg.SSE == 0
 *     raise virtual instruction exception
 *   else if privilege_mode == VU && senvcfg.SSE == 0
 *     raise virtual instruction exception
 *   else
 *     X(rd) = mem[X(rs1)]
 *     mem[X(rs1)] = X(rs2)
 *   endif
 *
 *   The model does not presently support H extension. The
 *   parts of algorithm dealing with VS and VU should be
 *   introduced when H extension is introduced.
 */
/* ****************************************************************** */
function ssamoswap_width_valid(size : word_width) -> bool = {
  match(size) {
    WORD   => true,
    DOUBLE => sizeof(xlen) >= 64,
    _      => false
  }
}
union clause ast = SSAMOSWAP_W : (bool, bool, regidx, regidx, regidx)
mapping clause encdec = SSAMOSWAP_W(aq, rl, rs2, rs1, rd)
      if extensionEnabled(Ext_Zicfiss)
  <-> 0b01001 @ bool_bits(aq) @ bool_bits(rl) @ rs2 @ rs1 @ 0b010 @ rd @ 0b0101111
      if extensionEnabled(Ext_Zicfiss)

union clause ast = SSAMOSWAP_D : (bool, bool, regidx, regidx, regidx)
mapping clause encdec = SSAMOSWAP_D(aq, rl, rs2, rs1, rd)
      if extensionEnabled(Ext_Zicfiss)
  <-> 0b01001 @ bool_bits(aq) @ bool_bits(rl) @ rs2 @ rs1 @ 0b011 @ rd @ 0b0101111
      if extensionEnabled(Ext_Zicfiss)

/* Architectural state for the Zicfiss extension. */
register ssp : xlenbits
mapping clause csr_name_map = 0x011 <-> "ssp"
