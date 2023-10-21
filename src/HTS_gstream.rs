use crate::{util::*, HTS_PStreamSet, HTS_Vocoder, HTS_error};

use crate::{
    HTS_PStreamSet_get_msd_flag, HTS_PStreamSet_get_nstream, HTS_PStreamSet_get_parameter,
    HTS_PStreamSet_get_total_frame, HTS_PStreamSet_get_vector_length, HTS_PStreamSet_is_msd,
    HTS_Vocoder_clear, HTS_Vocoder_initialize, HTS_Vocoder_synthesize, HTS_calloc, HTS_free,
};

#[derive(Clone)]
pub struct HTS_GStream {
    pub vector_length: size_t,
    pub par: *mut *mut libc::c_double,
}

#[derive(Clone)]
pub struct HTS_GStreamSet {
    pub total_nsample: size_t,
    pub total_frame: size_t,
    pub nstream: size_t,
    pub gstream: *mut HTS_GStream,
    pub gspeech: *mut libc::c_double,
}

pub fn HTS_GStreamSet_initialize() -> HTS_GStreamSet {
    HTS_GStreamSet {
        nstream: 0 as libc::c_int as size_t,
        total_frame: 0 as libc::c_int as size_t,
        total_nsample: 0 as libc::c_int as size_t,
        gstream: std::ptr::null_mut::<HTS_GStream>(),
        gspeech: std::ptr::null_mut::<libc::c_double>(),
    }
}

pub unsafe fn HTS_GStreamSet_create(
    gss: &mut HTS_GStreamSet,
    pss: &mut HTS_PStreamSet,
    stage: size_t,
    use_log_gain: bool,
    sampling_rate: size_t,
    fperiod: size_t,
    alpha: libc::c_double,
    beta: libc::c_double,
    stop: bool,
    volume: libc::c_double,
) -> bool {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut k: size_t = 0;
    let mut msd_frame: size_t = 0;
    let mut v: HTS_Vocoder = HTS_Vocoder {
        is_first: 0,
        stage: 0,
        gamma: 0.,
        use_log_gain: 0,
        fprd: 0,
        next: 0,
        gauss: 0,
        rate: 0.,
        pitch_of_curr_point: 0.,
        pitch_counter: 0.,
        pitch_inc_per_point: 0.,
        excite_ring_buff: std::ptr::null_mut::<libc::c_double>(),
        excite_buff_size: 0,
        excite_buff_index: 0,
        sw: 0,
        x: 0,
        freqt_buff: std::ptr::null_mut::<libc::c_double>(),
        freqt_size: 0,
        spectrum2en_buff: std::ptr::null_mut::<libc::c_double>(),
        spectrum2en_size: 0,
        r1: 0.,
        r2: 0.,
        s: 0.,
        postfilter_buff: std::ptr::null_mut::<libc::c_double>(),
        postfilter_size: 0,
        c: std::ptr::null_mut::<libc::c_double>(),
        cc: std::ptr::null_mut::<libc::c_double>(),
        cinc: std::ptr::null_mut::<libc::c_double>(),
        d1: std::ptr::null_mut::<libc::c_double>(),
        lsp2lpc_buff: std::ptr::null_mut::<libc::c_double>(),
        lsp2lpc_size: 0,
        gc2gc_buff: std::ptr::null_mut::<libc::c_double>(),
        gc2gc_size: 0,
    };
    let mut nlpf: size_t = 0 as libc::c_int as size_t;
    let mut lpf: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    if !(gss.gstream).is_null() || !(gss.gspeech).is_null() {
        HTS_error!(
            1 as libc::c_int,
            b"HTS_GStreamSet_create: HTS_GStreamSet is not initialized.\n\0" as *const u8
                as *const libc::c_char,
        );
        return false;
    }
    gss.nstream = HTS_PStreamSet_get_nstream(pss);
    gss.total_frame = HTS_PStreamSet_get_total_frame(pss);
    gss.total_nsample = fperiod * gss.total_frame;
    gss.gstream = HTS_calloc(
        gss.nstream,
        ::core::mem::size_of::<HTS_GStream>() as libc::c_ulong,
    ) as *mut HTS_GStream;
    i = 0 as libc::c_int as size_t;
    while i < gss.nstream {
        (*(gss.gstream).offset(i as isize)).vector_length =
            HTS_PStreamSet_get_vector_length(pss, i);
        let fresh0 = &mut (*(gss.gstream).offset(i as isize)).par;
        *fresh0 = HTS_calloc(
            gss.total_frame,
            ::core::mem::size_of::<*mut libc::c_double>() as libc::c_ulong,
        ) as *mut *mut libc::c_double;
        j = 0 as libc::c_int as size_t;
        while j < gss.total_frame {
            let fresh1 = &mut (*((*(gss.gstream).offset(i as isize)).par).offset(j as isize));
            *fresh1 = HTS_calloc(
                (*(gss.gstream).offset(i as isize)).vector_length,
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    gss.gspeech = HTS_calloc(
        gss.total_nsample,
        ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
    ) as *mut libc::c_double;
    i = 0 as libc::c_int as size_t;
    while i < gss.nstream {
        if HTS_PStreamSet_is_msd(pss, i) != 0 {
            j = 0 as libc::c_int as size_t;
            msd_frame = 0 as libc::c_int as size_t;
            while j < gss.total_frame {
                if HTS_PStreamSet_get_msd_flag(pss, i, j) != 0 {
                    k = 0 as libc::c_int as size_t;
                    while k < (*(gss.gstream).offset(i as isize)).vector_length {
                        *(*((*(gss.gstream).offset(i as isize)).par).offset(j as isize))
                            .offset(k as isize) =
                            HTS_PStreamSet_get_parameter(pss, i, msd_frame, k);
                        k = k.wrapping_add(1);
                    }
                    msd_frame = msd_frame.wrapping_add(1);
                } else {
                    k = 0 as libc::c_int as size_t;
                    while k < (*(gss.gstream).offset(i as isize)).vector_length {
                        *(*((*(gss.gstream).offset(i as isize)).par).offset(j as isize))
                            .offset(k as isize) = -1.0e+10f64;
                        k = k.wrapping_add(1);
                    }
                }
                j = j.wrapping_add(1);
            }
        } else {
            j = 0 as libc::c_int as size_t;
            while j < gss.total_frame {
                k = 0 as libc::c_int as size_t;
                while k < (*(gss.gstream).offset(i as isize)).vector_length {
                    *(*((*(gss.gstream).offset(i as isize)).par).offset(j as isize))
                        .offset(k as isize) = HTS_PStreamSet_get_parameter(pss, i, j, k);
                    k = k.wrapping_add(1);
                }
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    if gss.nstream != 2 as libc::c_int as size_t && gss.nstream != 3 as libc::c_int as size_t {
        HTS_error!(
            1 as libc::c_int,
            b"HTS_GStreamSet_create: The number of streams should be 2 or 3.\n\0" as *const u8
                as *const libc::c_char,
        );
        HTS_GStreamSet_clear(gss);
        return false;
    }
    if HTS_PStreamSet_get_vector_length(pss, 1 as libc::c_int as size_t)
        != 1 as libc::c_int as size_t
    {
        HTS_error!(
            1 as libc::c_int,
            b"HTS_GStreamSet_create: The size of lf0 static vector should be 1.\n\0" as *const u8
                as *const libc::c_char,
        );
        HTS_GStreamSet_clear(gss);
        return false;
    }
    if gss.nstream >= 3 as libc::c_int as size_t
        && (*(gss.gstream).offset(2 as libc::c_int as isize)).vector_length
            % 2 as libc::c_int as size_t
            == 0 as libc::c_int as size_t
    {
        HTS_error!(
            1 as libc::c_int,
            b"HTS_GStreamSet_create: The number of low-pass filter coefficient should be odd numbers.\0"
                as *const u8 as *const libc::c_char,
        );
        HTS_GStreamSet_clear(gss);
        return false;
    }
    HTS_Vocoder_initialize(
        &mut v,
        ((*(gss.gstream).offset(0 as libc::c_int as isize)).vector_length)
            .wrapping_sub(1 as libc::c_int as size_t),
        stage,
        use_log_gain as i8,
        sampling_rate,
        fperiod,
    );
    if gss.nstream >= 3 as libc::c_int as size_t {
        nlpf = (*(gss.gstream).offset(2 as libc::c_int as isize)).vector_length;
    }
    i = 0 as libc::c_int as size_t;
    while i < gss.total_frame && stop == false {
        j = i * fperiod;
        if gss.nstream >= 3 as libc::c_int as size_t {
            lpf = &mut *(*((*(gss.gstream).offset(2 as libc::c_int as isize)).par)
                .offset(i as isize))
            .offset(0 as libc::c_int as isize) as *mut libc::c_double;
        }
        HTS_Vocoder_synthesize(
            &mut v,
            ((*(gss.gstream).offset(0 as libc::c_int as isize)).vector_length)
                .wrapping_sub(1 as libc::c_int as size_t),
            *(*((*(gss.gstream).offset(1 as libc::c_int as isize)).par).offset(i as isize))
                .offset(0 as libc::c_int as isize),
            &mut *(*((*(gss.gstream).offset(0 as libc::c_int as isize)).par).offset(i as isize))
                .offset(0 as libc::c_int as isize),
            nlpf,
            lpf,
            alpha,
            beta,
            volume,
            &mut *(gss.gspeech).offset(j as isize),
        );
        i = i.wrapping_add(1);
    }
    HTS_Vocoder_clear(&mut v);
    // if !audio.is_null() {
    //     HTS_Audio_flush(audio);
    // }
    true
}

pub unsafe fn HTS_GStreamSet_get_total_nsamples(gss: &mut HTS_GStreamSet) -> size_t {
    gss.total_nsample
}

pub unsafe fn HTS_GStreamSet_get_total_frame(gss: &mut HTS_GStreamSet) -> size_t {
    gss.total_frame
}

pub unsafe fn HTS_GStreamSet_get_vector_length(
    gss: &mut HTS_GStreamSet,
    stream_index: size_t,
) -> size_t {
    (*(gss.gstream).offset(stream_index as isize)).vector_length
}

pub unsafe fn HTS_GStreamSet_get_speech(
    gss: &mut HTS_GStreamSet,
    sample_index: size_t,
) -> libc::c_double {
    *(gss.gspeech).offset(sample_index as isize)
}

pub unsafe fn HTS_GStreamSet_get_parameter(
    gss: &mut HTS_GStreamSet,
    stream_index: size_t,
    frame_index: size_t,
    vector_index: size_t,
) -> libc::c_double {
    *(*((*(gss.gstream).offset(stream_index as isize)).par).offset(frame_index as isize))
        .offset(vector_index as isize)
}

pub unsafe fn HTS_GStreamSet_clear(gss: &mut HTS_GStreamSet) {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    if !(gss.gstream).is_null() {
        i = 0 as libc::c_int as size_t;
        while i < gss.nstream {
            if !((*(gss.gstream).offset(i as isize)).par).is_null() {
                j = 0 as libc::c_int as size_t;
                while j < gss.total_frame {
                    HTS_free(
                        *((*(gss.gstream).offset(i as isize)).par).offset(j as isize)
                            as *mut libc::c_void,
                    );
                    j = j.wrapping_add(1);
                }
                HTS_free((*(gss.gstream).offset(i as isize)).par as *mut libc::c_void);
            }
            i = i.wrapping_add(1);
        }
        HTS_free(gss.gstream as *mut libc::c_void);
    }
    if !(gss.gspeech).is_null() {
        HTS_free(gss.gspeech as *mut libc::c_void);
    }
}
