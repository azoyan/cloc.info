--
-- PostgreSQL database dump
--

-- Dumped from database version 14.4
-- Dumped by pg_dump version 14.4

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: repository_info(text, text, text); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.repository_info(hn text, o text, n text) RETURNS TABLE(hostname text, owner text, repository_name text, branches text)
    LANGUAGE plpgsql
    AS $_$
BEGIN
   RETURN QUERY
    select * from branches where repository_id = (select id from repositories where hostname='$1' and owner='$2' and repository_name='$3');
    
END
$_$;


ALTER FUNCTION public.repository_info(hn text, o text, n text) OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: branches; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.branches (
    id bigint NOT NULL,
    repository_id bigint,
    name text NOT NULL,
    last_commit_sha text NOT NULL,
    scc_output bytea,
    size bigint
);


ALTER TABLE public.branches OWNER TO postgres;

--
-- Name: branches_view; Type: VIEW; Schema: public; Owner: postgres
--

CREATE VIEW public.branches_view AS
 SELECT branches.id,
    branches.repository_id,
    branches.name,
    branches.last_commit_sha,
    branches.size
   FROM public.branches;


ALTER TABLE public.branches_view OWNER TO postgres;

--
-- Name: statistic; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.statistic (
    id bigint NOT NULL,
    user_agent text,
    branch_id bigint,
    "time" timestamp with time zone
);


ALTER TABLE public.statistic OWNER TO postgres;

--
-- Name: recently_branches_view; Type: VIEW; Schema: public; Owner: postgres
--

CREATE VIEW public.recently_branches_view AS
 SELECT statistic.branch_id,
    (array_agg(statistic."time" ORDER BY statistic."time" DESC))[1] AS time
   FROM public.statistic
  GROUP BY statistic.branch_id;


ALTER TABLE public.recently_branches_view OWNER TO postgres;

--
-- Name: repositories; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.repositories (
    id bigint NOT NULL,
    hostname text NOT NULL,
    owner text NOT NULL,
    repository_name text NOT NULL,
    default_branch text NOT NULL
);


ALTER TABLE public.repositories OWNER TO postgres;

--
-- Name: repositories_view; Type: VIEW; Schema: public; Owner: postgres
--

CREATE VIEW public.repositories_view AS
 SELECT branches_view.id,
    repositories.hostname,
    repositories.owner,
    repositories.repository_name,
    repositories.default_branch,
    branches_view.name,
    branches_view.last_commit_sha,
    branches_view.size
   FROM (public.repositories
     JOIN public.branches_view ON ((repositories.id = branches_view.repository_id)));


ALTER TABLE public.repositories_view OWNER TO postgres;

--
-- Name: all_view; Type: VIEW; Schema: public; Owner: postgres
--

CREATE VIEW public.all_view AS
 SELECT repositories_view.id,
    repositories_view.hostname,
    repositories_view.owner,
    repositories_view.repository_name,
    repositories_view.default_branch,
    repositories_view.name,
    repositories_view.last_commit_sha,
    repositories_view.size,
    recently_branches_view.branch_id,
    recently_branches_view.time AS "time"
   FROM (public.repositories_view
     JOIN public.recently_branches_view ON ((repositories_view.id = recently_branches_view.branch_id)));


ALTER TABLE public.all_view OWNER TO postgres;

--
-- Name: branches_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.branches_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.branches_id_seq OWNER TO postgres;

--
-- Name: branches_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.branches_id_seq OWNED BY public.branches.id;


--
-- Name: largest_repositories; Type: VIEW; Schema: public; Owner: postgres
--

CREATE VIEW public.largest_repositories AS
 SELECT repositories.hostname,
    repositories.owner,
    repositories.repository_name,
    repositories.default_branch,
    branches_view.name,
    branches_view.last_commit_sha,
    branches_view.size
   FROM (public.repositories
     JOIN public.branches_view ON ((repositories.id = branches_view.repository_id)))
  ORDER BY branches_view.size DESC;


ALTER TABLE public.largest_repositories OWNER TO postgres;

--
-- Name: popular_branches; Type: VIEW; Schema: public; Owner: postgres
--

CREATE VIEW public.popular_branches AS
 SELECT count(*) AS count,
    statistic.branch_id
   FROM public.statistic
  GROUP BY statistic.branch_id
  ORDER BY (count(*)) DESC;


ALTER TABLE public.popular_branches OWNER TO postgres;

--
-- Name: popular_repositories; Type: VIEW; Schema: public; Owner: postgres
--

CREATE VIEW public.popular_repositories AS
 SELECT popular_branches.count,
    popular_branches.branch_id,
    repositories_view.id,
    repositories_view.hostname,
    repositories_view.owner,
    repositories_view.repository_name,
    repositories_view.default_branch,
    repositories_view.name,
    repositories_view.last_commit_sha,
    repositories_view.size
   FROM (public.popular_branches
     JOIN public.repositories_view ON ((repositories_view.id = popular_branches.branch_id)));


ALTER TABLE public.popular_repositories OWNER TO postgres;

--
-- Name: repositories_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.repositories_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.repositories_id_seq OWNER TO postgres;

--
-- Name: repositories_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.repositories_id_seq OWNED BY public.repositories.id;


--
-- Name: statistic_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.statistic_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.statistic_id_seq OWNER TO postgres;

--
-- Name: statistic_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.statistic_id_seq OWNED BY public.statistic.id;


--
-- Name: branches id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.branches ALTER COLUMN id SET DEFAULT nextval('public.branches_id_seq'::regclass);


--
-- Name: repositories id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.repositories ALTER COLUMN id SET DEFAULT nextval('public.repositories_id_seq'::regclass);


--
-- Name: statistic id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.statistic ALTER COLUMN id SET DEFAULT nextval('public.statistic_id_seq'::regclass);


--
-- Name: branches branches_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.branches
    ADD CONSTRAINT branches_pkey PRIMARY KEY (id);


--
-- Name: branches branches_repo_id_name_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.branches
    ADD CONSTRAINT branches_repo_id_name_key UNIQUE (repository_id, name);


--
-- Name: repositories repositories_hostname_owner_repository_name_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.repositories
    ADD CONSTRAINT repositories_hostname_owner_repository_name_key UNIQUE (hostname, owner, repository_name);


--
-- Name: repositories repositories_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.repositories
    ADD CONSTRAINT repositories_pkey PRIMARY KEY (id);


--
-- Name: branches branches_repo_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.branches
    ADD CONSTRAINT branches_repo_id_fkey FOREIGN KEY (repository_id) REFERENCES public.repositories(id);


--
-- PostgreSQL database dump complete
--

